use super::*;

impl Fake {
    pub(super) fn activate_target(&mut self, target: &u64) -> Result<(), &'static str> {
        let mut world = self.0.lock().unwrap();
        let fail = world.step("activate");
        world.foreground = Some(*target);
        if fail {
            Err("activation confirmation failed")
        } else {
            Ok(())
        }
    }

    pub(super) fn restore_focus_target(&mut self, focus: &u64) -> Result<(), &'static str> {
        let mut world = self.0.lock().unwrap();
        if world.step("restore_focus") {
            return Err("saved foreground is unavailable");
        }
        world.foreground = Some(*focus);
        Ok(())
    }

    pub(super) fn foreground_target(&mut self) -> Result<Option<u64>, &'static str> {
        let mut world = self.0.lock().unwrap();
        if world.step("foreground") {
            return Err("foreground unavailable");
        }
        Ok(world.foreground)
    }

    pub(super) fn release_focus_if_owned_target(
        &mut self,
        target: &u64,
    ) -> Result<(), &'static str> {
        let mut world = self.0.lock().unwrap();
        if world.step("release_focus") {
            return Err("focus release refused");
        }
        if world.foreground == Some(*target) {
            world.foreground = None;
        }
        Ok(())
    }
}
