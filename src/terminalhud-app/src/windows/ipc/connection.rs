use super::{
    super::{
        host::{UiMessage, UiSender},
        native::Handle,
        security::{self, Identity},
    },
    UI_TIMEOUT,
    accept::accept,
    read::read,
    write::write,
};
use crate::protocol::{self, REQUEST_LEN, ReplyCode};
use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};
use terminalhud_core::Command;
use windows_sys::Win32::{Foundation::*, System::Threading::*};
pub(super) fn handle_connection(
    pipe: &Handle,
    stop: &Handle,
    identity: &Identity,
    ui: &UiSender,
    mode: crate::mode::Mode,
) -> io::Result<bool> {
    accept(pipe.raw(), stop.raw())?;
    security::verify_peer(pipe, false, identity)?;
    let mut frame = [0; REQUEST_LEN];
    let count = read(pipe.raw(), &mut frame, Some(stop.raw()))?;
    let command = match protocol::decode_request(&frame[..count]) {
        Ok(command) => command,
        Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, error)),
    };
    let (sender, receiver) = mpsc::sync_channel(1);
    ui.send(UiMessage::Request {
        command,
        reply: Some(sender),
    })?;
    let deadline = Instant::now()
        + if mode == crate::mode::Mode::Real {
            Duration::from_secs(20)
        } else {
            UI_TIMEOUT
        };
    let reply = loop {
        if unsafe { WaitForSingleObject(stop.raw(), 0) } == WAIT_OBJECT_0 {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "server stopping",
            ));
        }
        match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(reply) => break reply,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "UI stopped"));
            }
            Err(_) if Instant::now() >= deadline => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "UI response deadline",
                ));
            }
            Err(_) => {}
        }
    };
    let quitting = command == Command::Quit && reply.code == ReplyCode::Ok;
    let result = write(pipe.raw(), &reply.encode(), Some(stop.raw())).and_then(|()| {
        // DisconnectNamedPipe discards unread bytes. A bounded client ACK proves
        // consumption without an unbounded synchronous FlushFileBuffers call.
        let mut ack = [0; 4];
        let count = read(pipe.raw(), &mut ack, Some(stop.raw()))?;
        if count != ack.len() || &ack != b"THA1" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid reply acknowledgement",
            ));
        }
        Ok(())
    });
    // Even a disconnected quit client must not strand a confirmed shutdown.
    if quitting {
        let _ = ui.send(UiMessage::QuitReplyWritten);
    }
    result.map(|_| quitting)
}
