//! Only the nonce-scoped fixture session is asked to exit, never a Terminal PID.
mod handshake;
use super::{Result, files::write_new, setup::Setup};
pub(super) use handshake::read as read_handshake;
use std::os::windows::io::{AsRawHandle, OwnedHandle};
use std::path::PathBuf;
use terminalhud_windows_terminal::SessionProof;
use windows_sys::Win32::{Foundation::WAIT_TIMEOUT, System::Threading::*};

pub(super) struct Session {
    task: PathBuf,
    pub _launcher: OwnedHandle,
    shell: Option<OwnedHandle>,
}
impl Session {
    pub(super) fn new(setup: &Setup, launcher: OwnedHandle) -> Self {
        Self {
            task: setup.task.clone(),
            _launcher: launcher,
            shell: None,
        }
    }
    pub(super) fn handshake(&mut self) -> Result<SessionProof> {
        let (proof, shell) = handshake::read(&self.task)?;
        self.shell = Some(shell);
        Ok(proof)
    }
    pub(super) fn alive(&self) -> bool {
        self.shell.as_ref().is_some_and(
            |process| unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } == WAIT_TIMEOUT,
        )
    }
    pub(super) fn request_exit(&self) -> Result<()> {
        write_new(
            &self.task.join("exit.request"),
            b"TER-9 explicit fixture cleanup\n",
        )
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if !self.task.join("exit.request").exists()
            && let Err(error) = self.request_exit()
        {
            eprintln!("FIXTURE CLEANUP FAILED: {error}");
        }
    }
}
