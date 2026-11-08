use super::fixture::Primary;
use std::process::Command as ProcessCommand;
use terminalhud_app::protocol;
use terminalhud_core::{
    Command,
    integration::{Availability, State, VisibleState},
};
#[test]
fn authenticated_cli_roundtrip_singleton_and_reply_before_quit() {
    let mut primary = Primary::start();
    let pid = primary.pid();
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_terminalhud"))
        .args([
            "--prototype",
            "--instance",
            primary.instance(),
            "visibility-toggle",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("mode=prototype"));
    assert!(stdout.contains(&format!("pid={pid}")));
    assert!(stdout.contains("Passive"));
    let duplicate = ProcessCommand::new(env!("CARGO_BIN_EXE_terminalhud-shell"))
        .args([
            "--prototype",
            "--headless",
            "--instance",
            primary.instance(),
        ])
        .output()
        .unwrap();
    assert!(duplicate.status.success());
    assert_eq!(primary.call(Command::Status).process_id, pid);
    assert_eq!(
        primary.call(Command::VisibilityToggle).status.state,
        Some(State::Hidden {
            return_state: VisibleState::Passive
        })
    );
    let reply = primary.call(Command::Quit);
    assert_eq!(reply.code, protocol::ReplyCode::Ok);
    assert_eq!(reply.status.availability, Availability::Unmanaged);
    primary.wait();
}
