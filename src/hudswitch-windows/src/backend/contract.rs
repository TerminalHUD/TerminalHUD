use super::*;

impl<L: Launcher> Backend for WindowsBackend<L> {
    type Target = Target;
    type Focus = Focus;
    type Original = Original;
    type Error = Error;

    fn capabilities(&self) -> Capabilities {
        Capabilities::ALL
    }

    fn acquire(&mut self, intent: VisibleState) -> Result<Acquired<Target, Original>, Error> {
        self.acquire_target(intent)
    }

    fn is_valid(&mut self, target: &Target) -> Result<bool, Error> {
        Ok(target.valid())
    }

    fn foreground(&mut self) -> Result<Option<Focus>, Error> {
        self.capture_foreground()
    }

    fn present(&mut self, target: &Target, presentation: Presentation) -> Result<(), Error> {
        self.present_target(target, presentation)
    }

    fn activate(&mut self, target: &Target) -> Result<(), Error> {
        self.activate_target(target)
    }

    fn restore_focus(&mut self, focus: &Focus) -> Result<(), Error> {
        self.restore_saved_focus(focus)
    }

    fn release_focus_if_owned(&mut self, target: &Target) -> Result<(), Error> {
        self.release_owned_focus(target)
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
        self.restore_target(target, original)
    }
}
