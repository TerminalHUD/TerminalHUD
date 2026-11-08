//! Win32 implementation of the portable HudSwitch backend contract.
//!
//! The caller owns the event/controller thread and supplies an intended-target
//! launcher. This backend has no permanent event loop and never searches for an
//! arbitrary existing user window. See `docs/windows-backend.md`.

#![cfg(windows)]

use hudswitch_core::{Acquired, Backend, Capabilities, Observation, Presentation, VisibleState};
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::process::Child;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::DataExchange::{GlobalAddAtomW, GlobalDeleteAtom};
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const LAYER_BITS: u32 = WS_EX_LAYERED | WS_EX_TRANSPARENT;
const WAIT: Duration = Duration::from_millis(500);
static NEXT_TOKEN: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub enum Error {
    Win32 { operation: &'static str, code: u32 },
    Launch(std::io::Error),
    InvalidTarget,
    AlreadyManaged,
    Unsupported(&'static str),
    FocusRefused,
    Timeout(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

fn last_error(operation: &'static str) -> Error {
    Error::Win32 {
        operation,
        code: unsafe { GetLastError() },
    }
}

/// A launcher-created window with a retained handle to its owning process.
/// Neither this receipt nor the managed target terminates its process on drop.
pub struct LaunchedWindow {
    hwnd: HWND,
    process: OwnedHandle,
    pid: u32,
    tid: u32,
}

impl LaunchedWindow {
    /// Claim a window created by the exact child process just launched.
    pub fn from_child(child: &Child, hwnd: HWND) -> Result<Self, Error> {
        let mut copy = null_mut();
        // SAFETY: Child owns a valid process handle; the output is adopted once.
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                child.as_raw_handle(),
                GetCurrentProcess(),
                &mut copy,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        } == 0
        {
            return Err(last_error("DuplicateHandle"));
        }
        let process = unsafe { OwnedHandle::from_raw_handle(copy) };
        Self::claim(hwnd, process)
    }

    /// Claim a dedicated window produced by a verified delegated/server launch.
    ///
    /// # Safety
    /// The caller must prove this specific window belongs to the isolated target
    /// it launched, not merely an existing window of the same application. The
    /// process handle must identify that window's actual owner. Passive launch
    /// must be nonactivating before this receipt is constructed. This trust
    /// boundary is for terminal/platform launch adapters, never arbitrary attach.
    pub unsafe fn from_verified_launch(hwnd: HWND, process: OwnedHandle) -> Result<Self, Error> {
        Self::claim(hwnd, process)
    }

    fn claim(hwnd: HWND, process: OwnedHandle) -> Result<Self, Error> {
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if tid == 0
            || pid == 0
            || unsafe { GetProcessId(process.as_raw_handle()) } != pid
            || unsafe { GetAncestor(hwnd, GA_ROOT) } != hwnd
            || unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } != WAIT_TIMEOUT
        {
            return Err(Error::InvalidTarget);
        }
        Ok(Self {
            hwnd,
            process,
            pid,
            tid,
        })
    }
}

/// Launch/acquire only an intended, dedicated target. Keep partial launch
/// cleanup in the launcher; a rejected receipt has had no style mutations.
pub trait Launcher {
    fn launch(&mut self, intent: VisibleState) -> Result<LaunchedWindow, Error>;
}
impl<F: FnMut(VisibleState) -> Result<LaunchedWindow, Error>> Launcher for F {
    fn launch(&mut self, intent: VisibleState) -> Result<LaunchedWindow, Error> {
        self(intent)
    }
}

struct IdentityAtom(u16);
impl IdentityAtom {
    fn key(&self) -> *const u16 {
        self.0 as usize as *const u16
    }
}
impl Drop for IdentityAtom {
    fn drop(&mut self) {
        // Integer-atom properties do not allocate an atom per target acquisition.
        unsafe {
            GlobalDeleteAtom(self.0);
        }
    }
}

/// Opaque Win32 target identity; validates process, thread, and acquisition token.
pub struct Target {
    window: LaunchedWindow,
    atom: Arc<IdentityAtom>,
    token: usize,
    original_layered: bool,
    needs_layer_init: Cell<bool>,
    changed_style: Cell<u32>,
    changed_topmost: Cell<bool>,
    changed_visibility: Cell<bool>,
}
impl Target {
    pub fn window_handle(&self) -> HWND {
        self.window.hwnd
    }
    fn valid(&self) -> bool {
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(self.window.hwnd, &mut pid) };
        tid == self.window.tid
            && pid == self.window.pid
            && unsafe { WaitForSingleObject(self.window.process.as_raw_handle(), 0) }
                == WAIT_TIMEOUT
            && unsafe { GetPropW(self.window.hwnd, self.atom.key()) } as usize == self.token
    }
    fn ensure(&self) -> Result<(), Error> {
        if self.valid() {
            Ok(())
        } else {
            Err(Error::InvalidTarget)
        }
    }
}
impl Drop for Target {
    fn drop(&mut self) {
        if self.valid() {
            unsafe {
                RemovePropW(self.window.hwnd, self.atom.key());
            }
        }
    }
}

pub struct Original {
    exstyle: u32,
    visible: bool,
}

/// Foreground identity is read-only; capturing it does not mark or mutate user windows.
pub struct Focus {
    hwnd: HWND,
    pid: u32,
    tid: u32,
    process: OwnedHandle,
}

/// Thread-confined backend, composed into the caller's authoritative event loop.
pub struct WindowsBackend<L> {
    launcher: L,
    atom: Arc<IdentityAtom>,
    _thread: PhantomData<Rc<()>>,
}
impl<L: Launcher> WindowsBackend<L> {
    pub fn new(launcher: L) -> Result<Self, Error> {
        let name: Vec<u16> = "HudSwitch.Acquisition.v1\0"
            .to_owned()
            .encode_utf16()
            .collect();
        let atom = unsafe { GlobalAddAtomW(name.as_ptr()) };
        if atom == 0 {
            return Err(last_error("GlobalAddAtomW"));
        }
        Ok(Self {
            launcher,
            atom: Arc::new(IdentityAtom(atom)),
            _thread: PhantomData,
        })
    }
}

fn exstyle(hwnd: HWND) -> Result<u32, Error> {
    unsafe {
        SetLastError(0);
    }
    let value = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    if value == 0 && unsafe { GetLastError() } != 0 {
        Err(last_error("GetWindowLongPtrW"))
    } else {
        Ok(value as u32)
    }
}

fn set_style(target: &Target, mask: u32, bits: u32) -> Result<(), Error> {
    target.ensure()?;
    let previous = exstyle(target.window.hwnd)?;
    let desired = (previous & !mask) | (bits & mask);
    let changed = (previous ^ desired) & mask;
    if changed == 0 {
        return Ok(());
    }
    // Record attempted ownership before the API, including partial/error cases.
    target
        .changed_style
        .set(target.changed_style.get() | changed);
    unsafe {
        SetLastError(0);
    }
    let value = unsafe { SetWindowLongPtrW(target.window.hwnd, GWL_EXSTYLE, desired as isize) };
    if value == 0 && unsafe { GetLastError() } != 0 {
        Err(last_error("SetWindowLongPtrW"))
    } else {
        Ok(())
    }
}

fn position(target: &Target, topmost: Option<bool>, visible: Option<bool>) -> Result<(), Error> {
    target.ensure()?;
    let before = exstyle(target.window.hwnd)?;
    let was_visible = unsafe { IsWindowVisible(target.window.hwnd) } != 0;
    let mut flags =
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_ASYNCWINDOWPOS;
    let after = match topmost {
        Some(value) => {
            if value != (before & WS_EX_TOPMOST != 0) {
                target.changed_topmost.set(true);
            }
            if value { HWND_TOPMOST } else { HWND_NOTOPMOST }
        }
        None => {
            flags |= SWP_NOZORDER;
            null_mut()
        }
    };
    if let Some(value) = visible
        && value != was_visible
    {
        target.changed_visibility.set(true);
    }
    if unsafe { SetWindowPos(target.window.hwnd, after, 0, 0, 0, 0, flags) } == 0 {
        return Err(last_error("SetWindowPos"));
    }
    if let Some(value) = visible {
        target.ensure()?;
        if unsafe {
            ShowWindowAsync(
                target.window.hwnd,
                if value { SW_SHOWNOACTIVATE } else { SW_HIDE },
            )
        } == 0
        {
            return Err(last_error("ShowWindowAsync"));
        }
    }
    wait_until(target, "window presentation", || {
        let style = exstyle(target.window.hwnd)?;
        Ok(
            topmost.is_none_or(|value| value == (style & WS_EX_TOPMOST != 0))
                && visible.is_none_or(|value| {
                    value == (unsafe { IsWindowVisible(target.window.hwnd) } != 0)
                }),
        )
    })
}

fn wait_until(
    target: &Target,
    operation: &'static str,
    mut condition: impl FnMut() -> Result<bool, Error>,
) -> Result<(), Error> {
    let deadline = Instant::now() + WAIT;
    loop {
        target.ensure()?;
        if condition()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(Error::Timeout(operation));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

impl<L: Launcher> Backend for WindowsBackend<L> {
    type Target = Target;
    type Focus = Focus;
    type Original = Original;
    type Error = Error;

    fn capabilities(&self) -> Capabilities {
        Capabilities::ALL
    }

    fn acquire(&mut self, intent: VisibleState) -> Result<Acquired<Target, Original>, Error> {
        let window = self.launcher.launch(intent)?;
        if intent == VisibleState::Passive && unsafe { GetForegroundWindow() } == window.hwnd {
            return Err(Error::Unsupported("launcher activated a Passive target"));
        }
        if !unsafe { GetPropW(window.hwnd, self.atom.key()) }.is_null() {
            return Err(Error::AlreadyManaged);
        }
        if unsafe { GetClassLongPtrW(window.hwnd, GCL_STYLE) } as u32 & (CS_OWNDC | CS_CLASSDC) != 0
        {
            return Err(Error::Unsupported(
                "window class cannot use layered click-through",
            ));
        }
        let original = Original {
            exstyle: exstyle(window.hwnd)?,
            visible: unsafe { IsWindowVisible(window.hwnd) } != 0,
        };
        let token = NEXT_TOKEN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value
                    .checked_add(1)
                    .filter(|value| *value <= u32::MAX as usize)
            })
            .map_err(|_| Error::Unsupported("acquisition token exhausted"))?;
        let token = usize::try_from((u64::from(std::process::id()) << 32) | token as u64)
            .map_err(|_| Error::Unsupported("64-bit Windows target required"))?;
        if unsafe { SetPropW(window.hwnd, self.atom.key(), token as HANDLE) } == 0 {
            return Err(last_error("SetPropW (check privilege compatibility)"));
        }
        let target = Target {
            window,
            atom: self.atom.clone(),
            token,
            original_layered: original.exstyle & WS_EX_LAYERED != 0,
            needs_layer_init: Cell::new(false),
            changed_style: Cell::new(0),
            changed_topmost: Cell::new(false),
            changed_visibility: Cell::new(false),
        };
        target.ensure()?;
        Ok(Acquired { target, original })
    }

    fn is_valid(&mut self, target: &Target) -> Result<bool, Error> {
        Ok(target.valid())
    }

    fn foreground(&mut self) -> Result<Option<Focus>, Error> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_null() {
            return Ok(None);
        }
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                pid,
            )
        };
        if tid == 0 || handle.is_null() {
            return Err(last_error("OpenProcess foreground identity"));
        }
        Ok(Some(Focus {
            hwnd,
            pid,
            tid,
            process: unsafe { OwnedHandle::from_raw_handle(handle) },
        }))
    }

    fn present(&mut self, target: &Target, presentation: Presentation) -> Result<(), Error> {
        target.ensure()?;
        if presentation == Presentation::Hidden {
            return position(target, None, Some(false));
        }
        let style = exstyle(target.window.hwnd)?;
        let mut mask = WS_EX_TRANSPARENT;
        let mut desired = 0;
        if presentation == Presentation::Passive {
            if target.original_layered && style & WS_EX_LAYERED == 0 {
                return Err(Error::Unsupported(
                    "target changed its renderer layering mode",
                ));
            }
            desired |= WS_EX_TRANSPARENT;
            if style & WS_EX_LAYERED == 0 {
                mask |= WS_EX_LAYERED;
                desired |= WS_EX_LAYERED;
                target.needs_layer_init.set(true);
            }
        } else if !target.original_layered && target.changed_style.get() & WS_EX_LAYERED != 0 {
            mask |= WS_EX_LAYERED;
        }
        set_style(target, mask, desired)?;
        if presentation == Presentation::Active {
            target.needs_layer_init.set(false);
        }
        if presentation == Presentation::Passive && target.needs_layer_init.get() {
            // Only initialize layering we added to an originally nonlayered window.
            // Existing layered/colorkey/per-pixel state is never rewritten.
            target.ensure()?;
            if unsafe { SetLayeredWindowAttributes(target.window.hwnd, 0, 255, LWA_ALPHA) } == 0 {
                return Err(last_error(
                    "SetLayeredWindowAttributes opaque initialization",
                ));
            }
            target.needs_layer_init.set(false);
        }
        position(target, Some(true), Some(true))
    }

    fn activate(&mut self, target: &Target) -> Result<(), Error> {
        target.ensure()?;
        if unsafe { IsWindowEnabled(target.window.hwnd) } == 0
            || unsafe { IsWindowVisible(target.window.hwnd) } == 0
        {
            return Err(Error::FocusRefused);
        }
        if unsafe { SetForegroundWindow(target.window.hwnd) } == 0 {
            return Err(Error::FocusRefused);
        }
        wait_until(target, "foreground activation", || {
            Ok(unsafe { GetForegroundWindow() } == target.window.hwnd)
        })
    }

    fn restore_focus(&mut self, focus: &Focus) -> Result<(), Error> {
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(focus.hwnd, &mut pid) };
        if pid != focus.pid
            || tid != focus.tid
            || unsafe { WaitForSingleObject(focus.process.as_raw_handle(), 0) } != WAIT_TIMEOUT
            || unsafe { IsWindowVisible(focus.hwnd) } == 0
            || unsafe { IsWindowEnabled(focus.hwnd) } == 0
        {
            return Err(Error::InvalidTarget);
        }
        if unsafe { SetForegroundWindow(focus.hwnd) } == 0 {
            return Err(Error::FocusRefused);
        }
        let deadline = Instant::now() + WAIT;
        while unsafe { GetForegroundWindow() } != focus.hwnd {
            if Instant::now() >= deadline {
                return Err(Error::Timeout("foreground restoration"));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    fn release_focus_if_owned(&mut self, target: &Target) -> Result<(), Error> {
        target.ensure()?;
        if unsafe { GetForegroundWindow() } != target.window.hwnd {
            return Ok(());
        }
        let visible = unsafe { IsWindowVisible(target.window.hwnd) } != 0;
        // Hiding asks Windows to choose the next foreground; do not force a
        // desktop/shell/arbitrary window when saved-focus restoration failed.
        position(target, None, Some(false))?;
        wait_until(target, "foreground release", || {
            Ok(unsafe { GetForegroundWindow() } != target.window.hwnd)
        })?;
        if visible {
            position(target, None, Some(true))?;
        }
        if unsafe { GetForegroundWindow() } == target.window.hwnd {
            Err(Error::FocusRefused)
        } else {
            Ok(())
        }
    }

    fn observe(&mut self, target: &Target) -> Result<Observation, Error> {
        target.ensure()?;
        let style = exstyle(target.window.hwnd)?;
        Ok(Observation {
            visible: unsafe { IsWindowVisible(target.window.hwnd) } != 0,
            topmost: style & WS_EX_TOPMOST != 0,
            click_through: style & LAYER_BITS == LAYER_BITS,
            focused: unsafe { GetForegroundWindow() } == target.window.hwnd,
        })
    }

    fn restore_original(&mut self, target: &Target, original: &Original) -> Result<(), Error> {
        target.ensure()?;
        let changed = target.changed_style.get();
        set_style(target, changed, original.exstyle)?;
        let topmost = target
            .changed_topmost
            .get()
            .then_some(original.exstyle & WS_EX_TOPMOST != 0);
        let visible = target.changed_visibility.get().then_some(original.visible);
        position(target, topmost, visible)?;
        if exstyle(target.window.hwnd)? & changed != original.exstyle & changed {
            return Err(Error::Timeout("original style restoration"));
        }
        target.ensure()?;
        if unsafe { RemovePropW(target.window.hwnd, target.atom.key()) } as usize != target.token {
            return Err(last_error("RemovePropW acquisition identity"));
        }
        Ok(())
    }
}
