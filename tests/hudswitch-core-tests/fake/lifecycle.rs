use super::*;

impl Fake {
    pub(super) fn is_valid_target(&mut self, target: &u64) -> Result<bool, &'static str> {
        let mut world = self.0.lock().unwrap();
        if world.step("validate") {
            return Err("identity cannot be validated");
        }
        Ok(world.windows.contains_key(target))
    }

    pub(super) fn restore_original_target(
        &mut self,
        target: &u64,
        original: &Original,
    ) -> Result<(), &'static str> {
        let mut world = self.0.lock().unwrap();
        let fail = world.step("restore_original");
        let window = world.windows.get_mut(target).ok_or("target closed")?;
        window.style = (window.style & !OWNED) | original.owned_style;
        window.visible = original.visible;
        if fail {
            Err("original restoration could not be confirmed")
        } else {
            Ok(())
        }
    }
}
