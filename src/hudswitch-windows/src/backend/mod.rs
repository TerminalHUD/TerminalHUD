use hudswitch_core::{Acquired, Backend, Capabilities, Observation, Presentation, VisibleState};
use std::cell::Cell;
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

use crate::Error;
use crate::error::last_error;

mod acquisition;
mod contract;
mod focus;
mod launch;
mod marker;
mod position;
mod presentation;
mod recovery;
mod restore;
pub use recovery::{RecoveryClaim, RecoveryWatch};
mod style;
mod wait;

use position::position;
use style::{exstyle, set_style};
use wait::wait_until;

const LAYER_BITS: u32 = WS_EX_LAYERED | WS_EX_TRANSPARENT;
const WAIT: Duration = Duration::from_millis(500);
static NEXT_TOKEN: AtomicUsize = AtomicUsize::new(1);

/// A launcher-created window with a retained handle to its owning process.
/// Neither this receipt nor the managed target terminates its process on drop.
pub struct LaunchedWindow {
    hwnd: HWND,
    process: OwnedHandle,
    pid: u32,
    tid: u32,
}

/// Launch/acquire only an intended, dedicated target. Keep partial launch
/// cleanup in the launcher; a rejected receipt has had no style mutations.
pub trait Launcher {
    fn launch(&mut self, intent: VisibleState) -> Result<LaunchedWindow, Error>;
    /// Register a recovery watcher before any presentation changes.
    fn enroll(&mut self, _claim: RecoveryClaim) -> Result<bool, Error> {
        Ok(false)
    }
}
impl<F: FnMut(VisibleState) -> Result<LaunchedWindow, Error>> Launcher for F {
    fn launch(&mut self, intent: VisibleState) -> Result<LaunchedWindow, Error> {
        self(intent)
    }
}

struct IdentityAtom(u16);

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
    recovery_enrolled: bool,
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

#[cfg(test)]
#[path = "../../../../tests/hudswitch-backend-tests/acquisition.rs"]
mod acquisition_tests;
