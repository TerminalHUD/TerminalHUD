//! Explicit simulation used to validate shell composition without a real target.

use std::convert::Infallible;
use terminalhud_core::integration::*;

#[derive(Default)]
pub struct PrototypeBackend {
    observed: Option<Observation>,
}

impl Backend for PrototypeBackend {
    type Target = ();
    type Focus = ();
    type Original = ();
    type Error = Infallible;
    fn capabilities(&self) -> Capabilities {
        Capabilities::ALL
    }
    fn acquire(&mut self, intent: VisibleState) -> Result<Acquired<(), ()>, Infallible> {
        self.observed = Some(Observation {
            visible: false,
            topmost: false,
            click_through: false,
            focused: intent == VisibleState::Active,
        });
        Ok(Acquired {
            target: (),
            original: (),
        })
    }
    fn is_valid(&mut self, _: &()) -> Result<bool, Infallible> {
        Ok(self.observed.is_some())
    }
    fn foreground(&mut self) -> Result<Option<()>, Infallible> {
        Ok(None)
    }
    fn present(&mut self, _: &(), state: Presentation) -> Result<(), Infallible> {
        self.observed = Some(Observation {
            visible: state != Presentation::Hidden,
            topmost: true,
            click_through: state == Presentation::Passive,
            focused: false,
        });
        Ok(())
    }
    fn activate(&mut self, _: &()) -> Result<(), Infallible> {
        self.observed.as_mut().unwrap().focused = true;
        Ok(())
    }
    fn restore_focus(&mut self, _: &()) -> Result<(), Infallible> {
        Ok(())
    }
    fn release_focus_if_owned(&mut self, _: &()) -> Result<(), Infallible> {
        if let Some(s) = &mut self.observed {
            s.focused = false;
        }
        Ok(())
    }
    fn observe(&mut self, _: &()) -> Result<Observation, Infallible> {
        Ok(self.observed.unwrap())
    }
    fn restore_original(&mut self, _: &(), _: &()) -> Result<(), Infallible> {
        self.observed = None;
        Ok(())
    }
}
