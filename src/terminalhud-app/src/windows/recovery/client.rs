use super::super::{
    ipc,
    native::{Handle, last},
    security::Identity,
};
use super::{read, write};
use hudswitch_windows::RecoveryClaim;
use std::{io, os::windows::io::AsRawHandle, time::Duration};
use windows_sys::Win32::{
    Foundation::WAIT_TIMEOUT,
    System::{
        Pipes::GetNamedPipeServerProcessId,
        Threading::{INFINITE, WaitForSingleObject},
    },
};

pub(in crate::windows) fn enroll(
    instance: &str,
    bridge: &std::os::windows::io::OwnedHandle,
    claim: RecoveryClaim,
) -> io::Result<()> {
    enroll_session(instance, bridge, claim).map(drop)
}
pub(in crate::windows) fn enroll_session(
    instance: &str,
    bridge: &std::os::windows::io::OwnedHandle,
    claim: RecoveryClaim,
) -> io::Result<Handle> {
    let expected =
        unsafe { windows_sys::Win32::System::Threading::GetProcessId(bridge.as_raw_handle()) };
    if expected == 0 || unsafe { WaitForSingleObject(bridge.as_raw_handle(), 0) } != WAIT_TIMEOUT {
        return Err(io::Error::other("session bridge is no longer alive"));
    }
    let pipe = ipc::connect(&Identity::current()?, instance, Duration::from_secs(2))?;
    let mut actual = 0;
    if unsafe { GetNamedPipeServerProcessId(pipe.raw(), &mut actual) } == 0 {
        return Err(last("recovery server PID"));
    }
    if actual != expected
        || unsafe { WaitForSingleObject(bridge.as_raw_handle(), 0) } != WAIT_TIMEOUT
    {
        return Err(io::Error::other(
            "recovery server is not the retained bridge",
        ));
    }
    let frame = format!(
        "{} {} {} {} {} {} {}",
        claim.hwnd,
        claim.pid,
        claim.tid,
        claim.token,
        claim.exstyle,
        u8::from(claim.visible),
        super::creation_time(unsafe {
            windows_sys::Win32::System::Threading::GetCurrentProcess()
        })?
    );
    write::write(pipe.raw(), frame.as_bytes(), None)?;
    let mut reply = [0; 2];
    if read::read(pipe.raw(), &mut reply, None)? != 2 || &reply != b"OK" {
        return Err(io::Error::other("recovery enrollment rejected"));
    }
    Ok(pipe)
}
pub(in crate::windows) fn completion(
    pipe: &Handle,
    bridge: &std::os::windows::io::OwnedHandle,
    key: &str,
) -> io::Result<()> {
    let mut frame = [0u8; 160];
    let count = read::read_with_timeout(
        pipe.raw(),
        &mut frame,
        Some(bridge.as_raw_handle()),
        INFINITE,
    )?;
    if &frame[..count] != format!("DONE {key}").as_bytes() {
        return Err(io::Error::other("invalid session completion frame"));
    }
    Ok(())
}
pub(in crate::windows) fn release(pipe: &Handle, key: &str) -> io::Result<()> {
    write::write(pipe.raw(), format!("RELS {key}").as_bytes(), None)?;
    let mut reply = [0u8; 160];
    let count = read::read(pipe.raw(), &mut reply, None)?;
    if &reply[..count] != format!("CLEAR {key}").as_bytes() {
        return Err(io::Error::other(
            "recovery watcher could not confirm target release",
        ));
    }
    Ok(())
}
