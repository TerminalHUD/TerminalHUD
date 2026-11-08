use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use terminalhud_core::{ShellController, integration::*};
pub(super) struct Fake {
    state: Observation,
    calls: Arc<AtomicUsize>,
    fail_quit: Arc<AtomicBool>,
}
impl Backend for Fake {
    type Target = ();
    type Focus = ();
    type Original = ();
    type Error = &'static str;
    fn capabilities(&self) -> Capabilities {
        Capabilities::ALL
    }
    fn acquire(&mut self, _: VisibleState) -> Result<Acquired<(), ()>, Self::Error> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(Acquired {
            target: (),
            original: (),
        })
    }
    fn is_valid(&mut self, _: &()) -> Result<bool, Self::Error> {
        Ok(true)
    }
    fn foreground(&mut self) -> Result<Option<()>, Self::Error> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }
    fn present(&mut self, _: &(), presentation: Presentation) -> Result<(), Self::Error> {
        self.state = Observation {
            visible: presentation != Presentation::Hidden,
            topmost: true,
            click_through: presentation == Presentation::Passive,
            focused: false,
        };
        Ok(())
    }
    fn activate(&mut self, _: &()) -> Result<(), Self::Error> {
        self.state.focused = true;
        Ok(())
    }
    fn restore_focus(&mut self, _: &()) -> Result<(), Self::Error> {
        Ok(())
    }
    fn release_focus_if_owned(&mut self, _: &()) -> Result<(), Self::Error> {
        self.state.focused = false;
        Ok(())
    }
    fn observe(&mut self, _: &()) -> Result<Observation, Self::Error> {
        Ok(self.state)
    }
    fn restore_original(&mut self, _: &(), _: &()) -> Result<(), Self::Error> {
        if self.fail_quit.load(Ordering::Relaxed) {
            Err("restore failed")
        } else {
            Ok(())
        }
    }
}
pub(super) fn controller() -> (ShellController<Fake>, Arc<AtomicUsize>, Arc<AtomicBool>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let fail = Arc::new(AtomicBool::new(false));
    (
        ShellController::new(Fake {
            state: Observation {
                visible: false,
                topmost: false,
                click_through: false,
                focused: false,
            },
            calls: calls.clone(),
            fail_quit: fail.clone(),
        }),
        calls,
        fail,
    )
}
