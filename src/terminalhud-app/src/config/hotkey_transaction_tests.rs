use super::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use terminalhud_core::config::Candidate;

#[derive(Default)]
struct State {
    active: BTreeMap<i32, Shortcut>,
    calls: usize,
    fail: Option<Shortcut>,
    fail_cleanup: bool,
}
struct Fake(Rc<RefCell<State>>);
impl HotkeyPlatform for Fake {
    type Guard = i32;
    fn register(&mut self, id: i32, shortcut: &Shortcut) -> Result<i32, RegisterError<i32>> {
        let mut state = self.0.borrow_mut();
        state.calls += 1;
        if state.fail.as_ref() == Some(shortcut) {
            return Err(RegisterError {
                message: "conflict".into(),
                code: 1409,
                remaining: None,
            });
        }
        state.active.insert(id, shortcut.clone());
        Ok(id)
    }
    fn unregister(&mut self, id: &mut i32) -> Result<(), String> {
        let mut state = self.0.borrow_mut();
        if state.fail_cleanup {
            return Err("cleanup refused".into());
        }
        state.active.remove(id);
        Ok(())
    }
}
fn setup() -> (Rc<RefCell<State>>, HotkeyTransaction<Fake>, Configuration) {
    let state = Rc::new(RefCell::new(State::default()));
    let mut keys = HotkeyTransaction::new(Fake(state.clone()));
    let config = Configuration::default();
    let prepared = keys.prepare(&config).unwrap();
    keys.commit(prepared, &config).unwrap();
    (state, keys, config)
}
fn candidate(interaction: &str, visibility: &str) -> Configuration {
    Configuration::validate(Candidate {
        toggle_interaction: interaction.into(),
        toggle_visibility: visibility.into(),
        ..Default::default()
    })
    .unwrap()
}
#[test]
fn unchanged_and_swapped_shortcuts_reuse_ids_and_change_actions() {
    let (state, mut keys, current) = setup();
    let original = keys.actions().to_vec();
    let prepared = keys.prepare(&current).unwrap();
    keys.commit(prepared, &current).unwrap();
    assert_eq!(state.borrow().calls, 2);
    let swapped = candidate(
        &current.toggle_visibility().to_string(),
        &current.toggle_interaction().to_string(),
    );
    let prepared = keys.prepare(&swapped).unwrap();
    keys.commit(prepared, &swapped).unwrap();
    assert_eq!(state.borrow().calls, 2);
    assert_eq!(
        keys.actions(),
        &[
            (original[1].0, Command::InteractionToggle),
            (original[0].0, Command::VisibilityToggle)
        ]
    );
}
#[test]
fn failed_registration_cleans_new_keys_and_preserves_old_actions() {
    let (state, mut keys, _) = setup();
    let original = keys.actions().to_vec();
    let old = state.borrow().active.clone();
    let next = candidate("Ctrl+A", "Ctrl+B");
    state.borrow_mut().fail = Some(Shortcut::parse("Ctrl+B").unwrap());
    assert!(keys.prepare(&next).err().unwrap().contains("conflict"));
    assert_eq!(state.borrow().active, old);
    assert_eq!(keys.actions(), original);
    assert_eq!(keys.error_code, 1409);
}
#[test]
fn failed_cleanup_is_reported_and_its_id_is_not_reused() {
    let (state, mut keys, _) = setup();
    state.borrow_mut().fail = Some(Shortcut::parse("Ctrl+B").unwrap());
    state.borrow_mut().fail_cleanup = true;
    assert!(
        keys.prepare(&candidate("Ctrl+A", "Ctrl+B"))
            .err()
            .unwrap()
            .contains("cleanup refused")
    );
    let stranded = *state.borrow().active.keys().max().unwrap();
    state.borrow_mut().fail = None;
    state.borrow_mut().fail_cleanup = false;
    let prepared = keys.prepare(&candidate("Ctrl+C", "Ctrl+D")).unwrap();
    assert!(!prepared.0.iter().any(|entry| entry.id == stranded));
    assert!(keys.cancel(prepared).is_empty());
}
