use super::*;

impl World {
    pub(crate) fn step(&mut self, operation: &str) -> bool {
        self.calls.push(operation.to_owned());
        if self.failures.front().copied() == Some(operation) {
            self.failures.pop_front();
            true
        } else {
            false
        }
    }

    pub(crate) fn target(&self) -> u64 {
        self.next_target
    }

    pub(crate) fn observation(&self) -> Observation {
        let target = self.target();
        let window = self.windows[&target];
        Observation {
            visible: window.visible,
            topmost: window.style & TOPMOST != 0,
            click_through: window.style & CLICK_THROUGH != 0,
            focused: self.foreground == Some(target),
        }
    }

    pub(crate) fn fail(&mut self, operations: &[&'static str]) {
        self.failures.extend(operations);
    }
}
