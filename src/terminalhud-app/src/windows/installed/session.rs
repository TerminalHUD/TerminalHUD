//! Retained authenticated completion channel for the current target epoch.
use crate::windows::{native::Handle, recovery};
use std::{
    io,
    os::windows::io::OwnedHandle,
    sync::{Arc, mpsc},
    thread,
};

pub(super) struct Session {
    pipe: Arc<Handle>,
    key: String,
    received: mpsc::Receiver<io::Result<()>>,
    completed: bool,
}
impl Session {
    pub fn new(pipe: Handle, bridge: Arc<OwnedHandle>, key: String) -> io::Result<Self> {
        let pipe = Arc::new(pipe);
        let worker_pipe = pipe.clone();
        let worker_key = key.clone();
        let (sender, received) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("terminalhud-session-completion".into())
            .spawn(move || {
                let _ = sender.send(recovery::completion(&worker_pipe, &bridge, &worker_key));
            })?;
        Ok(Self {
            pipe,
            key,
            received,
            completed: false,
        })
    }
    pub fn completion(&mut self) -> io::Result<bool> {
        if !self.completed {
            match self.received.try_recv() {
                Ok(result) => {
                    result?;
                    self.completed = true;
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(false),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(io::Error::other("session completion worker stopped"));
                }
            }
        }
        Ok(true)
    }
    pub fn release(&self) -> io::Result<()> {
        recovery::release(&self.pipe, &self.key)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        // A replaced target must not leave its old completion read alive.
        unsafe {
            windows_sys::Win32::System::IO::CancelIoEx(self.pipe.raw(), std::ptr::null());
        }
    }
}
