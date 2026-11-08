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
        assert_eq!(decode_request(&encode_request(command)), Ok(command));
    }
}
#[test]
fn unconfirmed_and_absent_status_stay_distinct() {
    let mut reply = Reply {
        code: ReplyCode::TransitionFailed,
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
