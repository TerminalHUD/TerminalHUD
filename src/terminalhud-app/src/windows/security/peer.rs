use super::{
    super::native::{Handle, last},
    Identity,
};
use std::io;
use windows_sys::Win32::System::Pipes::*;
pub fn verify_peer(pipe: &Handle, server: bool, expected: &Identity) -> io::Result<()> {
    let mut pid = 0;
    let ok = unsafe {
        if server {
            GetNamedPipeServerProcessId(pipe.raw(), &mut pid)
        } else {
            GetNamedPipeClientProcessId(pipe.raw(), &mut pid)
        }
    };
    if ok == 0 {
        return Err(last("named-pipe peer PID"));
    }
    let actual = Identity::process(pid)?;
    if actual != *expected {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "pipe peer user/session does not match",
        ));
    }
    Ok(())
}
