use super::*;

impl Fake {
    pub(super) fn observe_target(&mut self, _: &u64) -> Result<Observation, &'static str> {
        let mut world = self.0.lock().unwrap();
        if world.step("observe") {
            return Err("observation unavailable");
        }
        if world.failures.front() == Some(&"mismatch") {
            world.failures.pop_front();
            return Ok(Observation {
                visible: true,
                topmost: true,
                click_through: true,
                focused: true,
            });
        }
        Ok(world.observation())
    }
}
