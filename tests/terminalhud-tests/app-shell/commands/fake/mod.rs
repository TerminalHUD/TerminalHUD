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
mod backend;

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
