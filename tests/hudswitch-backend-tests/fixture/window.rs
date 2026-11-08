use super::*;

impl Fixture {
    pub(crate) fn hwnd(&self) -> HWND {
        self.hwnd as HWND
    }
    pub(crate) fn claim(&self) -> Result<LaunchedWindow, Error> {
        LaunchedWindow::from_child(&self.child, self.hwnd())
    }
    pub(crate) fn focus(&self) {
        assert_ne!(
            unsafe { SetForegroundWindow(self.hwnd()) },
            0,
            "Windows refused foreground control. Run this explicit desktop test from a foreground native PowerShell; no foreground policy is bypassed."
        );
        wait(|| unsafe { GetForegroundWindow() } == self.hwnd());
    }
    pub(crate) fn message(&self, message: u32) -> usize {
        let mut result = 0;
        assert_ne!(
            unsafe {
                SendMessageTimeoutW(
                    self.hwnd(),
                    message,
                    0,
                    0,
                    SMTO_ABORTIFHUNG,
                    500,
                    &mut result,
                )
            },
            0
        );
        result
    }
    pub(crate) fn style(&self) -> u32 {
        (unsafe { GetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE) }) as u32
    }
}
