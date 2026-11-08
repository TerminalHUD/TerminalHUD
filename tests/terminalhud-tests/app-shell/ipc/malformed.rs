use super::fixture::Primary;
use std::{thread, time::Duration};
use terminalhud_app::windows::{ipc, security::Identity};
use terminalhud_core::Command;
#[test]
fn malformed_and_stalled_clients_do_not_mutate_or_block_future_clients() {
    let mut primary = Primary::start();
    let identity = Identity::current().unwrap();
    primary.call(Command::VisibilityToggle);
    let before = primary.call(Command::Status);
    assert!(ipc::exchange(&identity, primary.instance(), b"bad").is_err());
    let after = primary.call(Command::Status);
    assert_eq!(after.sequence, before.sequence);
    assert_eq!(after.status.state, before.status.state);
    let stalled = ipc::connect(&identity, primary.instance(), Duration::from_secs(1)).unwrap();
    thread::sleep(Duration::from_millis(1800));
    drop(stalled);
    assert_eq!(primary.call(Command::Status).sequence, before.sequence);
    primary.call(Command::Quit);
    primary.wait();
}
