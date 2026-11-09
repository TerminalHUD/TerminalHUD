use super::*;
use terminalhud_core::{
    Command,
    integration::{Availability, Status},
};

#[test]
fn malformed_frames_cannot_become_commands() {
    for bad in [
        vec![],
        vec![0; 8],
        b"THP1\x01\x01\0\0".to_vec(),
        b"THP1\x09\0\0\0".to_vec(),
        b"THP1\x01\0\0\0extra".to_vec(),
    ] {
        assert!(decode_request(&bad).is_err());
    }
    for command in [
        Command::InteractionToggle,
        Command::VisibilityToggle,
        Command::Passive,
        Command::Status,
        Command::Quit,
    ] {
        assert_eq!(
            decode_request(&encode_request(command)),
            Ok(Request::Command(command))
        );
    }
}
#[test]
fn unconfirmed_and_absent_status_stay_distinct() {
    let mut reply = Reply {
        code: ReplyCode::TransitionFailed,
        mode: crate::mode::Mode::Prototype,
        hotkey_error: 0,
        status: Status {
            availability: Availability::Managed,
            state: None,
            target_id: None,
        },
        focus_warning: false,
        sequence: 1,
        menu_commands: 0,
        process_id: 123,
        host_console: false,
    };
    assert_eq!(
        Reply::decode(&reply.encode()).unwrap().status.availability,
        Availability::Managed
    );
    reply.status.availability = Availability::Absent;
    assert_eq!(
        Reply::decode(&reply.encode()).unwrap().status.availability,
        Availability::Absent
    );
    let mut invalid = reply.encode();
    invalid[7] = 1;
    assert!(Reply::decode(&invalid).is_err());
    let mut invalid = reply.encode();
    invalid[5] = 0;
    assert!(Reply::decode(&invalid).is_err());
}

#[test]
fn real_mode_and_native_conflicts_survive_reply_without_becoming_prototype_success() {
    let reply = Reply {
        code: ReplyCode::TransitionFailed,
        mode: crate::mode::Mode::Real,
        hotkey_error: 1409,
        status: Status {
            availability: Availability::Managed,
            state: None,
            target_id: None,
        },
        focus_warning: false,
        sequence: 4,
        menu_commands: 0,
        process_id: 123,
        host_console: false,
    };
    let decoded = Reply::decode(&reply.encode()).unwrap();
    assert_eq!(decoded.mode, crate::mode::Mode::Real);
    assert_eq!(decoded.hotkey_error, 1409);
    assert!(decoded.display().contains("mode=real hotkey_error=1409"));
    assert_eq!(decoded.status.state, None);
    let mut unknown = reply.encode();
    unknown[5] = 3;
    assert!(Reply::decode(&unknown).is_err());
}

#[test]
fn reload_has_its_own_opcode_without_changing_commands() {
    for (command, opcode) in [
        (Command::InteractionToggle, 1),
        (Command::VisibilityToggle, 2),
        (Command::Passive, 3),
        (Command::Status, 4),
        (Command::Quit, 5),
    ] {
        assert_eq!(encode_request(command)[4], opcode);
    }
    let frame = encode_request(Request::ReloadConfiguration);
    assert_eq!(frame[4], 6);
    assert_eq!(decode_request(&frame), Ok(Request::ReloadConfiguration));
}

#[test]
fn pending_launch_is_an_explicit_success_reply() {
    let reply = Reply {
        code: ReplyCode::RequiresRelaunch,
        mode: crate::mode::Mode::Real,
        hotkey_error: 0,
        status: Status {
            availability: Availability::Absent,
            state: None,
            target_id: None,
        },
        focus_warning: false,
        sequence: 1,
        menu_commands: 0,
        process_id: 123,
        host_console: false,
    };
    assert_eq!(
        Reply::decode(&reply.encode()).unwrap().code,
        ReplyCode::RequiresRelaunch
    );
}
