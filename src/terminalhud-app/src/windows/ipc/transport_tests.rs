//! Actual authenticated message pipe, delayed reply and acknowledgement.
use super::*;
use crate::{
    mode::Mode,
    protocol::{Reply, ReplyCode},
    windows::security::Identity,
};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use terminalhud_core::{
    Command,
    integration::{Availability, Status},
};
fn response() -> Reply {
    Reply {
        code: ReplyCode::TransitionFailed,
        mode: Mode::Real,
        hotkey_error: 1409,
        status: Status {
            availability: Availability::Managed,
            state: None,
            target_id: None,
        },
        focus_warning: false,
        sequence: 1,
        menu_commands: 0,
        process_id: std::process::id(),
        host_console: false,
    }
}
#[test]
fn real_reply_wait_handles_acquisition_delay_then_ack_and_reports_disconnect_ambiguity() {
    for disconnect in [false, true] {
        let identity = Identity::current().unwrap();
        let instance = format!(
            "delay-{:x}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let server = Server::bind(&identity, &instance).unwrap();
        let worker = thread::spawn(move || {
            accept::accept(server.pipe.raw(), server.stop.raw()).unwrap();
            let mut request = [0u8; crate::protocol::REQUEST_LEN];
            read::read(server.pipe.raw(), &mut request, None).unwrap();
            if disconnect {
                return;
            }
            thread::sleep(Duration::from_millis(1700));
            write::write(server.pipe.raw(), &response().encode(), None).unwrap();
            let mut ack = [0u8; 4];
            assert_eq!(read::read(server.pipe.raw(), &mut ack, None).unwrap(), 4);
            assert_eq!(&ack, b"THA1");
        });
        let reply = client::exchange_with_timeout(
            &identity,
            &instance,
            &crate::protocol::encode_request(Command::InteractionToggle),
            22_000,
        );
        if disconnect {
            assert!(reply.unwrap_err().to_string().contains("ambiguous"));
        } else {
            let reply = reply.unwrap();
            assert_eq!(reply.mode, Mode::Real);
            assert_eq!(reply.code, ReplyCode::TransitionFailed);
            assert_eq!(reply.status.state, None);
        }
        worker.join().unwrap();
    }
}
