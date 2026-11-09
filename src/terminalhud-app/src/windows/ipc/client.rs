use super::{
    super::{
        native::{Handle, last},
        security::{self, Identity},
    },
    read::read_with_timeout,
    write::write,
};
use crate::protocol::{self, REPLY_LEN, Reply};
use std::{
    io, ptr, thread,
    time::{Duration, Instant},
};
use terminalhud_core::Command;
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*, System::Pipes::*};
pub fn connect(identity: &Identity, instance: &str, timeout: Duration) -> io::Result<Handle> {
    let name = identity.pipe_name(instance)?;
    let deadline = Instant::now() + timeout;
    loop {
        let raw = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED | SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
                ptr::null_mut(),
            )
        };
        if raw != INVALID_HANDLE_VALUE {
            let pipe = unsafe { Handle::from_raw(raw, "CreateFile pipe")? };
            security::verify_peer(&pipe, true, identity)?;
            let mode = PIPE_READMODE_MESSAGE;
            if unsafe { SetNamedPipeHandleState(pipe.raw(), &mode, ptr::null(), ptr::null()) } == 0
            {
                return Err(last("SetNamedPipeHandleState"));
            }
            return Ok(pipe);
        }
        let error = unsafe { GetLastError() };
        if error != ERROR_PIPE_BUSY && error != ERROR_FILE_NOT_FOUND {
            return Err(io::Error::from_raw_os_error(error as i32));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "shell not ready/running",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn exchange(identity: &Identity, instance: &str, frame: &[u8]) -> io::Result<Reply> {
    let timeout = if instance == crate::mode::REAL_INSTANCE {
        22_000
    } else {
        super::IO_TIMEOUT
    };
    exchange_with_timeout(identity, instance, frame, timeout)
}
pub(super) fn exchange_with_timeout(
    identity: &Identity,
    instance: &str,
    frame: &[u8],
    timeout: u32,
) -> io::Result<Reply> {
    if frame.len() > crate::protocol::REQUEST_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "request exceeds fixed frame",
        ));
    }
    let pipe = connect(identity, instance, Duration::from_secs(2))?;
    write(pipe.raw(), frame, None)?;
    let mut response = [0; REPLY_LEN];
    let count = read_with_timeout(pipe.raw(), &mut response, None, timeout).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("{error}; command outcome may be ambiguous; inspect status before retrying"),
        )
    })?;
    let reply = Reply::decode(&response[..count]).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{error}; command outcome may be ambiguous; inspect status before retrying"),
        )
    })?;
    write(pipe.raw(), b"THA1", None)?;
    Ok(reply)
}
pub fn call(instance: &str, command: Command) -> io::Result<Reply> {
    exchange(
        &Identity::current()?,
        instance,
        &protocol::encode_request(command),
    )
}
