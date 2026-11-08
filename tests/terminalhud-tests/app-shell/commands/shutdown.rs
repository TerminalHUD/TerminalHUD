use super::fake::controller;
use std::sync::atomic::Ordering;
use terminalhud_core::{Command, integration::*};
#[test]
fn failed_quit_does_not_authorize_process_exit_and_can_be_retried() {
    let (mut host, _, fail) = controller();
    host.execute(Command::InteractionToggle).unwrap();
    fail.store(true, Ordering::Relaxed);
    assert!(host.execute(Command::Quit).is_err());
    assert_eq!(host.status().availability, Availability::Managed);
    fail.store(false, Ordering::Relaxed);
    let outcome = host.execute(Command::Quit).unwrap();
    assert!(outcome.should_exit);
    assert_eq!(outcome.status.availability, Availability::Unmanaged);
    assert!(host.execute(Command::Quit).unwrap().should_exit);
    assert!(!host.execute(Command::Status).unwrap().should_exit);
}
