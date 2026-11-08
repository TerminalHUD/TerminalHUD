use super::*;

impl Fake {
    pub(super) fn acquire_target(
        &mut self,
        intent: VisibleState,
    ) -> Result<Acquired<u64, Original>, &'static str> {
        let mut world = self.0.lock().unwrap();
        let operation = match intent {
            VisibleState::Active => "acquire.active",
            VisibleState::Passive => "acquire.passive",
        };
        if world.step(operation) {
            return Err("acquire failed without taking ownership");
        }
        world.next_target += 1;
        let target = world.target();
        let window = Window {
            visible: true,
            style: OTHER_STYLE,
        };
        let original = Original {
            owned_style: window.style & OWNED,
            visible: window.visible,
        };
        world.calls.push("snapshot".into());
        world.windows.insert(target, window);
        if intent == VisibleState::Active {
            world.foreground = Some(target);
        }
        Ok(Acquired { target, original })
    }
}
