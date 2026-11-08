use super::*;

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

    pub(super) fn claim(hwnd: HWND, process: OwnedHandle) -> Result<Self, Error> {
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        let window = Self {
            hwnd,
            process,
            pid,
            tid,
        };
        window.ensure_current()?;
        Ok(window)
    }

    pub(super) fn has_current_identity(&self) -> bool {
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(self.hwnd, &mut pid) };
        tid != 0
            && pid != 0
            && tid == self.tid
            && pid == self.pid
            && unsafe { GetProcessId(self.process.as_raw_handle()) } == self.pid
            && unsafe { GetAncestor(self.hwnd, GA_ROOT) } == self.hwnd
            && unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } == WAIT_TIMEOUT
    }

    pub(super) fn ensure_current(&self) -> Result<(), Error> {
        if self.has_current_identity() {
            Ok(())
        } else {
            Err(Error::InvalidTarget)
        }
    }
}
