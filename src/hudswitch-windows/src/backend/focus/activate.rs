use super::super::*;

impl<L: Launcher> WindowsBackend<L> {
    pub(in crate::backend) fn activate_target(&mut self, target: &Target) -> Result<(), Error> {
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
}
