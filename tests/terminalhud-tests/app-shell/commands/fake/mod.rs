use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use terminalhud_core::{ShellController, integration::*};
pub(super) struct Fake {
    state: Observation,
    calls: Arc<AtomicUsize>,
    fail_quit: Arc<AtomicBool>,
    creation: Arc<Creation>,
}
#[derive(Default)]
pub(super) struct Creation {
    pub(super) intents: Mutex<Vec<VisibleState>>,
    pub(super) stale: AtomicBool,
}
pub(super) struct Fixture {
    pub(super) host: ShellController<Fake>,
    pub(super) calls: Arc<AtomicUsize>,
    pub(super) fail: Arc<AtomicBool>,
    pub(super) creation: Arc<Creation>,
}
mod backend;

pub(super) fn controller() -> (ShellController<Fake>, Arc<AtomicUsize>, Arc<AtomicBool>) {
    let fixture = fixture();
    (fixture.host, fixture.calls, fixture.fail)
}
pub(super) fn fixture() -> Fixture {
    let calls = Arc::new(AtomicUsize::new(0));
    let fail = Arc::new(AtomicBool::new(false));
    let creation = Arc::new(Creation::default());
    Fixture {
        host: ShellController::new(Fake {
            state: Observation {
                visible: false,
                topmost: false,
                click_through: false,
                focused: false,
            },
            calls: calls.clone(),
            fail_quit: fail.clone(),
            creation: creation.clone(),
        }),
        calls,
        fail,
        creation,
    }
}
