use super::super::*;

impl<L: Launcher> WindowsBackend<L> {
    pub(in crate::backend) fn release_owned_focus(&mut self, target: &Target) -> Result<(), Error> {
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
}
