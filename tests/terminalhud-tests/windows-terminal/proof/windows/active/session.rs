//! Typed ownership of the normal adapter launch and exact nonce fixture.
use super::super::{Result, files::write_new, session::read_handshake, setup::Setup};
use std::{
    os::windows::io::{AsRawHandle, OwnedHandle},
    path::PathBuf,
};
use terminalhud_windows_terminal::{SessionProof, native::LauncherProcess};
use windows_sys::Win32::{Foundation::WAIT_TIMEOUT, System::Threading::WaitForSingleObject};
pub(super) struct ActiveSession {
    task: PathBuf,
    pub(super) launcher: LauncherProcess,
    shell: Option<OwnedHandle>,
}
impl ActiveSession {
    pub(super) fn new(setup: &Setup, launcher: LauncherProcess) -> Self {
        Self {
            task: setup.task.clone(),
            launcher,
            shell: None,
        }
    }
    pub(super) fn handshake(&mut self) -> Result<SessionProof> {
        let (proof, shell) = read_handshake(&self.task)?;
        self.shell = Some(shell);
        Ok(proof)
    }
    pub(super) fn alive(&self) -> bool {
        self.shell.as_ref().is_some_and(
            |shell| unsafe { WaitForSingleObject(shell.as_raw_handle(), 0) } == WAIT_TIMEOUT,
        )
    }
    pub(super) fn exit(&self) -> Result<()> {
        if !self.task.join("exit.request").exists() {
            write_new(
                &self.task.join("exit.request"),
                b"TER-9 owned Active fixture cleanup\n",
            )?;
        }
        Ok(())
    }
}
impl Drop for ActiveSession {
    fn drop(&mut self) {
        if let Err(error) = self.exit() {
            eprintln!("ACTIVE_CLEANUP_FAILED {error}");
        }
    }
}
