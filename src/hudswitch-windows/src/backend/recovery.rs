//! Independent watcher for an explicitly enrolled acquisition.
use super::*;

#[cfg(test)]
thread_local! { static FAIL_AFTER_INPUT: Cell<bool> = const { Cell::new(false) }; }
#[cfg(test)]
pub(super) fn fail_after_input_once() {
    FAIL_AFTER_INPUT.set(true);
}

/// Native claim sent to a trusted watcher while the acquiring process is alive.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryClaim {
    pub hwnd: usize,
    pub pid: u32,
    pub tid: u32,
    pub token: usize,
    pub exstyle: u32,
    pub visible: bool,
}
impl RecoveryClaim {
    pub(super) fn new(window: &LaunchedWindow, token: usize, original: &Original) -> Self {
        Self {
            hwnd: window.hwnd as usize,
            pid: window.pid,
            tid: window.tid,
            token,
            exstyle: original.exstyle,
            visible: original.visible,
        }
    }
}

/// Retains the target process epoch and property atom independently of the GUI.
pub struct RecoveryWatch {
    target: Target,
    original: Original,
}
impl RecoveryWatch {
    /// Open a claim received from the authenticated acquisition owner.
    pub fn open(claim: RecoveryClaim) -> Result<Self, Error> {
        if claim.token == 0 {
            return Err(Error::InvalidTarget);
        }
        let handle = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                claim.pid,
            )
        };
        if handle.is_null() {
            return Err(last_error("OpenProcess recovery target"));
        }
        let process = unsafe { OwnedHandle::from_raw_handle(handle) };
        let name: Vec<u16> = "HudSwitch.Acquisition.v1\0".encode_utf16().collect();
        let atom = unsafe { GlobalAddAtomW(name.as_ptr()) };
        if atom == 0 {
            return Err(last_error("GlobalAddAtomW recovery"));
        }
        let target = Target {
            window: LaunchedWindow {
                hwnd: claim.hwnd as HWND,
                process,
                pid: claim.pid,
                tid: claim.tid,
            },
            atom: Arc::new(IdentityAtom(atom)),
            token: claim.token,
            original_layered: claim.exstyle & WS_EX_LAYERED != 0,
            needs_layer_init: Cell::new(false),
            changed_style: Cell::new(0),
            changed_topmost: Cell::new(false),
            changed_visibility: Cell::new(false),
            recovery_enrolled: true,
        };
        target.ensure()?;
        Ok(Self {
            target,
            original: Original {
                exstyle: claim.exstyle,
                visible: claim.visible,
            },
        })
    }

    /// Restore critical input styles only while the exact claim still exists.
    /// A missing/changed claim is reported as InvalidTarget and never modified.
    pub fn restore(&self) -> Result<(), Error> {
        self.target.ensure()?;
        set_style(&self.target, LAYER_BITS, self.original.exstyle)?;
        if exstyle(self.target.window.hwnd)? & LAYER_BITS != self.original.exstyle & LAYER_BITS {
            return Err(Error::Timeout("recovery input styles"));
        }
        #[cfg(test)]
        if FAIL_AFTER_INPUT.replace(false) {
            return Err(Error::Unsupported("injected recovery position failure"));
        }
        // Do not issue z-order or visibility requests when already restored.
        // Pending asynchronous requests can still race this observation.
        let current_style = exstyle(self.target.window.hwnd)?;
        let topmost = self.original.exstyle & WS_EX_TOPMOST != 0;
        let visible = unsafe { IsWindowVisible(self.target.window.hwnd) } != 0;
        let restore_topmost = ((current_style & WS_EX_TOPMOST != 0) != topmost).then_some(topmost);
        let restore_visibility =
            (visible != self.original.visible).then_some(self.original.visible);
        if restore_topmost.is_some() || restore_visibility.is_some() {
            position(&self.target, restore_topmost, restore_visibility)?;
        }
        self.target.ensure()?;
        self.target.remove_own_marker()
    }
}
