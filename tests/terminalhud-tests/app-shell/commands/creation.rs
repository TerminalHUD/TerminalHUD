use super::fake::fixture;
use std::sync::atomic::Ordering;
use terminalhud_core::{Command, integration::*};

#[test]
fn either_shortcut_creates_active_instead_of_passive() {
    for command in [Command::InteractionToggle, Command::VisibilityToggle] {
        let mut fixture = fixture();
        let outcome = fixture.host.execute(command).unwrap();
        assert_eq!(outcome.status.state, Some(State::Active));
        assert_eq!(
            *fixture.creation.intents.lock().unwrap(),
            [VisibleState::Active]
        );
    }
}

#[test]
fn stale_target_visibility_toggle_recreates_active_with_a_new_identity() {
    let mut fixture = fixture();
    let old = fixture
        .host
        .execute(Command::InteractionToggle)
        .unwrap()
        .status;
    fixture.creation.stale.store(true, Ordering::Relaxed);
    let replacement = fixture
        .host
        .execute(Command::VisibilityToggle)
        .unwrap()
        .status;
    assert_ne!(replacement.target_id, old.target_id);
    assert_eq!(replacement.state, Some(State::Active));
    assert_eq!(
        *fixture.creation.intents.lock().unwrap(),
        [VisibleState::Active, VisibleState::Active]
    );
}

#[test]
fn passive_and_status_do_not_create_an_absent_target() {
    let mut fixture = fixture();
    for command in [Command::Passive, Command::Status] {
        let outcome = fixture.host.execute(command).unwrap();
        assert_eq!(outcome.status.availability, Availability::Absent);
    }
    assert!(fixture.creation.intents.lock().unwrap().is_empty());
}

#[test]
fn existing_passive_target_retains_reversible_hidden_memory() {
    let mut fixture = fixture();
    fixture.host.execute(Command::VisibilityToggle).unwrap();
    fixture.host.execute(Command::Passive).unwrap();
    assert_eq!(
        fixture
            .host
            .execute(Command::VisibilityToggle)
            .unwrap()
            .status
            .state,
        Some(State::Hidden {
            return_state: VisibleState::Passive
        })
    );
    assert_eq!(
        fixture
            .host
            .execute(Command::VisibilityToggle)
            .unwrap()
            .status
            .state,
        Some(State::Passive)
    );
    assert_eq!(fixture.creation.intents.lock().unwrap().len(), 1);
}
