use super::super::{
    native::{Handle, last},
    security::{self, Descriptor, Identity},
};
use super::IO_TIMEOUT;
use super::{accept, read, write};
use hudswitch_windows::{RecoveryClaim, RecoveryWatch};
use std::{
    io, ptr,
    sync::{Arc, mpsc},
    thread,
};
use windows_sys::Win32::{
    Foundation::*,
    Storage::FileSystem::*,
    System::{Pipes::*, Threading::*},
};

/// Development watcher remains detached and follows only the GUI lifetime.
pub(in crate::windows) fn start(instance: String) -> io::Result<()> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("terminalhud-recovery".into())
        .spawn(move || match bind(&instance) {
            Ok(pipe) => {
                let _ = sender.send(Ok(()));
                let result = event().and_then(|stop| serve_detached(pipe, stop));
                if let Err(error) = result {
                    eprintln!("TerminalHUD recovery: {error}");
                }
            }
            Err(error) => {
                let _ = sender.send(Err(error));
            }
        })?;
    receiver.recv().map_err(io::Error::other)?
}
/// Installed bridge stays alive until its UI has serialized target release.
pub(in crate::windows) struct SessionServer {
    finished: Arc<Handle>,
    done: mpsc::Receiver<io::Result<()>>,
    #[cfg(test)]
    sent: Arc<std::sync::atomic::AtomicBool>,
}
impl SessionServer {
    #[cfg(test)]
    pub(super) fn completion_written(&self) -> Arc<std::sync::atomic::AtomicBool> {
        self.sent.clone()
    }
    pub fn finish(self) -> io::Result<()> {
        if unsafe { SetEvent(self.finished.raw()) } == 0 {
            eprintln!("TerminalHUD could not signal session completion; waiting for owner exit");
        }
        match self.done.recv() {
            Ok(result) => result,
            Err(_) => {
                eprintln!(
                    "TerminalHUD recovery worker stopped unexpectedly; bridge remains available"
                );
                loop {
                    thread::park();
                }
            }
        }
    }
}
pub(in crate::windows) fn start_session(
    instance: String,
    key: String,
) -> io::Result<SessionServer> {
    let finished = Arc::new(event()?);
    let worker_finished = finished.clone();
    #[cfg(test)]
    let sent = Arc::new(std::sync::atomic::AtomicBool::new(false));
    #[cfg(test)]
    let worker_sent = sent.clone();
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("terminalhud-recovery".into())
        .spawn(move || match bind(&instance) {
            Ok(pipe) => {
                let _ = ready_tx.send(Ok(()));
                let _ = done_tx.send(serve_session(
                    pipe,
                    &worker_finished,
                    &key,
                    #[cfg(test)]
                    &worker_sent,
                ));
            }
            Err(error) => {
                let _ = ready_tx.send(Err(error));
            }
        })?;
    ready_rx.recv().map_err(io::Error::other)??;
    Ok(SessionServer {
        finished,
        done: done_rx,
        #[cfg(test)]
        sent,
    })
}
fn event() -> io::Result<Handle> {
    unsafe {
        Handle::from_raw(
            CreateEventW(ptr::null(), 1, 0, ptr::null()),
            "CreateEvent recovery",
        )
    }
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
fn open_claim(pipe: &Handle, stop: &Handle) -> io::Result<(Handle, RecoveryWatch)> {
    accept::accept(pipe.raw(), stop.raw())?;
    security::verify_peer(pipe, false, &Identity::current()?)?;
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
    if (claim.token as u64 >> 32) != u64::from(gui_pid)
        || unsafe { WaitForSingleObject(gui.raw(), 0) } != WAIT_TIMEOUT
        || super::creation_time(gui.raw())? != gui_created
    {
        return Err(io::Error::other("recovery GUI identity mismatch"));
    }
    let watch = RecoveryWatch::open(claim).map_err(io::Error::other)?;
    Ok((gui, watch))
}
fn serve_detached(pipe: Handle, stop: Handle) -> io::Result<()> {
    let (gui, watch) = open_claim(&pipe, &stop)?;
    write::write(pipe.raw(), b"OK", None)?;
    if unsafe { WaitForSingleObject(gui.raw(), INFINITE) } != WAIT_OBJECT_0 {
        return Err(last("wait for recovery GUI"));
    }
    restore(&watch)
}
fn serve_session(
    pipe: Handle,
    finished: &Handle,
    key: &str,
    #[cfg(test)] sent: &std::sync::atomic::AtomicBool,
) -> io::Result<()> {
    let (gui, watch) = match open_claim(&pipe, finished) {
        Ok(value) => value,
        Err(_) if unsafe { WaitForSingleObject(finished.raw(), 0) } == WAIT_OBJECT_0 => {
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    // If acknowledgement is ambiguous, retain the watcher until GUI exit.
    if write::write(pipe.raw(), b"OK", None).is_err() {
        return wait_and_restore(&gui, &watch);
    }
    let handles = [gui.raw(), finished.raw()];
    match unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE) } {
        WAIT_OBJECT_0 => restore_or_hold(&watch),
        result if result == WAIT_OBJECT_0 + 1 => {
            if write::write(pipe.raw(), format!("DONE {key}").as_bytes(), None).is_err() {
                return wait_and_restore(&gui, &watch);
            }
            #[cfg(test)]
            sent.store(true, std::sync::atomic::Ordering::Release);
            loop {
                let mut ack = [0u8; 160];
                match read::read_with_timeout(pipe.raw(), &mut ack, Some(gui.raw()), INFINITE) {
                    Ok(count) if &ack[..count] == format!("RELS {key}").as_bytes() => {
                        if matches!(watch.confirm_released(), Ok(true)) {
                            let _ =
                                write::write(pipe.raw(), format!("CLEAR {key}").as_bytes(), None);
                            return Ok(());
                        }
                        let _ = write::write(pipe.raw(), format!("HOLD {key}").as_bytes(), None);
                        return wait_and_restore(&gui, &watch);
                    }
                    Err(error) if error.kind() == io::ErrorKind::TimedOut => continue,
                    _ => return wait_and_restore(&gui, &watch),
                }
            }
        }
        _ => wait_and_restore(&gui, &watch),
    }
}
fn wait_and_restore(gui: &Handle, watch: &RecoveryWatch) -> io::Result<()> {
    if unsafe { WaitForSingleObject(gui.raw(), INFINITE) } != WAIT_OBJECT_0 {
        eprintln!("TerminalHUD cannot confirm GUI exit; bridge remains available");
        loop {
            thread::park();
        }
    }
    restore_or_hold(watch)
}
fn restore_or_hold(watch: &RecoveryWatch) -> io::Result<()> {
    match restore(watch) {
        Ok(()) => Ok(()),
        Err(error) => {
            eprintln!("TerminalHUD recovery incomplete: {error}; bridge remains available");
            loop {
                thread::park();
            }
        }
    }
}
fn restore(watch: &RecoveryWatch) -> io::Result<()> {
    match watch.restore() {
        Ok(()) => Ok(()),
        Err(hudswitch_windows::Error::InvalidTarget) => {
            if matches!(watch.confirm_released(), Ok(true)) {
                Ok(())
            } else {
                Err(io::Error::other(
                    "owned target identity or restoration is uncertain",
                ))
            }
        }
        Err(error) => Err(io::Error::other(error)),
    }
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
