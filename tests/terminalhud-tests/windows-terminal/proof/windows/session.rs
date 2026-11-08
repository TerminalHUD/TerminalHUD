//! Only the nonce-scoped fixture session is asked to exit, never a Terminal PID.
use super::{Result, files::write_new, setup::Setup};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
use terminalhud_windows_terminal::{Guid, SessionProof, native::LauncherProcess};
use windows_sys::Win32::{Foundation::WAIT_TIMEOUT, System::Threading::*};

pub(super) struct Session {
    task: PathBuf,
    pub _launcher: LauncherProcess,
    shell: Option<OwnedHandle>,
}
impl Session {
    pub(super) fn new(setup: &Setup, launcher: LauncherProcess) -> Self {
        Self {
            task: setup.task.clone(),
            _launcher: launcher,
            shell: None,
        }
    }
    pub(super) fn handshake(&mut self) -> Result<SessionProof> {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if let Ok(bytes) = fs::read(self.task.join("session-proof.json"))
                && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
            {
                let proof = SessionProof {
                    nonce: Guid::parse(value["nonce"].as_str().ok_or("handshake nonce missing")?)?,
                    profile: Guid::parse(
                        value["profile"]
                            .as_str()
                            .ok_or("handshake profile missing")?,
                    )?,
                    session: Guid::parse(
                        value["session"]
                            .as_str()
                            .ok_or("handshake session missing")?,
                    )?,
                    shell_pid: u32::try_from(
                        value["shellPid"].as_u64().ok_or("handshake PID missing")?,
                    )?,
                };
                let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, proof.shell_pid) };
                if handle.is_null() {
                    return Err("cannot retain owned fixture process".into());
                }
                self.shell = Some(unsafe { OwnedHandle::from_raw_handle(handle) });
                return Ok(proof);
            }
            if Instant::now() >= deadline {
                return Err("fixture handshake timed out".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
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
