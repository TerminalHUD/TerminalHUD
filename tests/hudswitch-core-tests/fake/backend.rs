use super::*;

impl Backend for Fake {
    type Target = u64;
    type Focus = u64;
    type Original = Original;
    type Error = &'static str;

    fn capabilities(&self) -> Capabilities {
        self.0.lock().unwrap().capabilities
    }

    fn acquire(&mut self, intent: VisibleState) -> Result<Acquired<u64, Original>, Self::Error> {
        self.acquire_target(intent)
    }

    fn present(&mut self, target: &u64, presentation: Presentation) -> Result<(), Self::Error> {
        self.present_target(target, presentation)
    }

    fn activate(&mut self, target: &u64) -> Result<(), Self::Error> {
        self.activate_target(target)
    }

    fn restore_focus(&mut self, focus: &u64) -> Result<(), Self::Error> {
        self.restore_focus_target(focus)
    }

    fn foreground(&mut self) -> Result<Option<u64>, Self::Error> {
        self.foreground_target()
    }

    fn release_focus_if_owned(&mut self, target: &u64) -> Result<(), Self::Error> {
        self.release_focus_if_owned_target(target)
    }

    fn is_valid(&mut self, target: &u64) -> Result<bool, Self::Error> {
        self.is_valid_target(target)
    }

    fn restore_original(&mut self, target: &u64, original: &Original) -> Result<(), Self::Error> {
        self.restore_original_target(target, original)
    }

    fn observe(&mut self, target: &u64) -> Result<Observation, Self::Error> {
        self.observe_target(target)
    }
}
