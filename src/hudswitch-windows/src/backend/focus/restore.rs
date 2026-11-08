use super::super::*;

impl<L: Launcher> WindowsBackend<L> {
    pub(in crate::backend) fn restore_saved_focus(&mut self, focus: &Focus) -> Result<(), Error> {
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
}
