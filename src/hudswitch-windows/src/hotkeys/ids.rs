use super::{HotkeyBinding, HotkeyFailure};
use std::{cell::RefCell, collections::BTreeSet};

thread_local! {
    static LIVE: RefCell<BTreeSet<i32>> = const { RefCell::new(BTreeSet::new()) };
}

pub(super) fn reserve(bindings: &[HotkeyBinding]) -> Result<(), HotkeyFailure> {
    LIVE.with(|live| {
        let mut live = live.borrow_mut();
        for &binding in bindings {
            if live.contains(&binding.id) {
                return Err(HotkeyFailure::Invalid {
                    binding,
                    reason: "hotkey ID already owned on this thread",
                });
            }
        }
        live.extend(bindings.iter().map(|binding| binding.id));
        Ok(())
    })
}

pub(super) fn release(id: i32) {
    // At thread exit the OS releases thread hotkeys; TLS may already be gone.
    let _ = LIVE.try_with(|live| live.borrow_mut().remove(&id));
}
