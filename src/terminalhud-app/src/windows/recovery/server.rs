use super::super::{
    native::{Handle, last},
    security::{self, Descriptor, Identity},
};
use super::{accept, read, write};
use hudswitch_windows::{RecoveryClaim, RecoveryWatch};
use std::{io, ptr, sync::mpsc, thread};
use windows_sys::Win32::{
    Foundation::*,
    Storage::FileSystem::*,
    System::{Pipes::*, Threading::*},
};

pub(in crate::windows) fn start(instance: String) -> io::Result<()> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("terminalhud-recovery".into())
        .spawn(move || {
            let result = bind(&instance);
            match result {
                Ok(pipe) => {
                    let _ = sender.send(Ok(()));
                    if let Err(error) = serve(pipe) {
                        eprintln!("TerminalHUD recovery: {error}");
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(error));
                }
            }
        })?;
    receiver.recv().map_err(io::Error::other)?
}
fn bind(instance: &str) -> io::Result<Handle> {
    let identity = Identity::current()?;
    let name = identity.pipe_name(instance)?;
    let descriptor = Descriptor::for_user(&identity)?;
    unsafe {
        Handle::from_raw(
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                256,
                256,
                IO_TIMEOUT,
                &descriptor.attributes(),
            ),
            "CreateNamedPipe recovery",
        )
    }
}
use super::IO_TIMEOUT;
fn serve(pipe: Handle) -> io::Result<()> {
    // One enrollment belongs to this one bridge/session; no arbitrary attach.
    let stop = unsafe {
        Handle::from_raw(
            CreateEventW(ptr::null(), 1, 0, ptr::null()),
            "CreateEvent recovery",
        )?
    };
    accept::accept(pipe.raw(), stop.raw())?;
    security::verify_peer(&pipe, false, &Identity::current()?)?;
    let mut gui_pid = 0;
    if unsafe { GetNamedPipeClientProcessId(pipe.raw(), &mut gui_pid) } == 0 {
        return Err(last("recovery client PID"));
    }
    let gui = unsafe {
        Handle::from_raw(
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                gui_pid,
            ),
            "OpenProcess recovery GUI",
        )?
    };
    let mut bytes = [0; 256];
    let count = read::read(pipe.raw(), &mut bytes, None)?;
    let (claim, gui_created) = parse(&bytes[..count])?;
    // Token's high half identifies the exact acquiring GUI process.
    if (claim.token as u64 >> 32) != u64::from(gui_pid)
        || unsafe { WaitForSingleObject(gui.raw(), 0) } != WAIT_TIMEOUT
        || super::creation_time(gui.raw())? != gui_created
    {
        return Err(io::Error::other("recovery GUI identity mismatch"));
    }
    let watch = RecoveryWatch::open(claim).map_err(io::Error::other)?;
    write::write(pipe.raw(), b"OK", None)?;
    if unsafe { WaitForSingleObject(gui.raw(), INFINITE) } != WAIT_OBJECT_0 {
        return Err(last("wait for recovery GUI"));
    }
    match watch.restore() {
        Ok(()) => eprintln!("TerminalHUD recovered owned input styles; visibility is best effort"),
        Err(hudswitch_windows::Error::InvalidTarget) => {
            eprintln!("TerminalHUD recovery claim absent or target changed; recovery not confirmed")
        }
        Err(error) => {
            return Err(io::Error::other(format!(
                "input style recovery incomplete: {error}"
            )));
        }
    }
    Ok(())
}
fn parse(bytes: &[u8]) -> io::Result<(RecoveryClaim, u64)> {
    let text = std::str::from_utf8(bytes).map_err(io::Error::other)?;
    let values: Vec<u64> = text
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(io::Error::other)?;
    if values.len() != 7 || values[5] > 1 {
        return Err(io::Error::other("invalid recovery claim"));
    }
    Ok((
        RecoveryClaim {
            hwnd: usize::try_from(values[0]).map_err(io::Error::other)?,
            pid: u32::try_from(values[1]).map_err(io::Error::other)?,
            tid: u32::try_from(values[2]).map_err(io::Error::other)?,
            token: usize::try_from(values[3]).map_err(io::Error::other)?,
            exstyle: u32::try_from(values[4]).map_err(io::Error::other)?,
            visible: values[5] != 0,
        },
        values[6],
    ))
}
