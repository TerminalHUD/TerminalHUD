use super::*;

impl Fake {
    pub(super) fn present_target(
        &mut self,
        target: &u64,
        presentation: Presentation,
    ) -> Result<(), &'static str> {
        let mut world = self.0.lock().unwrap();
        let operation = match presentation {
            Presentation::Active => "present.active",
            Presentation::Passive => "present.passive",
            Presentation::Hidden => "present.hidden",
        };
        let fail = world.step(operation);
        // Failures may occur after a partial mutation, as real backend calls can.
        let window = world.windows.get_mut(target).ok_or("target closed")?;
        match presentation {
            Presentation::Active => {
                window.visible = true;
                window.style = (window.style & !OWNED) | TOPMOST;
            }
            Presentation::Passive => {
                window.visible = true;
                window.style |= OWNED;
            }
            Presentation::Hidden => window.visible = false,
        }
        if fail {
            Err("presentation failed after mutation")
        } else {
            Ok(())
        }
    }
}
