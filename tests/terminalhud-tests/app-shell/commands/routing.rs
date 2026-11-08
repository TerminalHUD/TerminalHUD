use super::fake::controller;
use std::sync::atomic::Ordering;
use terminalhud_core::{Command, integration::*};
#[test]
fn status_is_read_only_and_commands_preserve_hidden_memory() {
    let (mut host, calls, _) = controller();
    assert_eq!(
        host.execute(Command::Status).unwrap().status.availability,
        Availability::Absent
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        host.execute(Command::InteractionToggle)
            .unwrap()
            .status
            .state,
        Some(State::Active)
    );
    let before = calls.load(Ordering::Relaxed);
    assert_eq!(
        host.execute(Command::Status).unwrap().status.state,
        Some(State::Active)
    );
    assert_eq!(calls.load(Ordering::Relaxed), before);
    assert_eq!(
        host.execute(Command::VisibilityToggle)
            .unwrap()
            .status
            .state,
        Some(State::Hidden {
            return_state: VisibleState::Active
        })
    );
    assert_eq!(
        host.execute(Command::VisibilityToggle)
            .unwrap()
            .status
            .state,
        Some(State::Active)
    );
    assert_eq!(
        host.execute(Command::Passive).unwrap().status.state,
        Some(State::Passive)
    );
}
