use super::{Command, REAL_INSTANCE, Result, ipc};
use std::{
    fs,
    os::windows::io::{AsRawHandle, OwnedHandle},
    path::{Path, PathBuf},
};
use terminalhud_app::{
    mode::Mode,
    protocol::{Reply, ReplyCode},
    windows::security::Identity,
};
use windows_sys::Win32::{
    Foundation::*,
    System::{Pipes::WaitNamedPipeW, Threading::*},
};
pub(super) fn require_no_owner() -> Result<()> {
    let name = Identity::current()?.pipe_name(REAL_INSTANCE)?;
    if unsafe { WaitNamedPipeW(name.as_ptr(), 1) } != 0
        || unsafe { GetLastError() } != ERROR_FILE_NOT_FOUND
    {
        return Err("real app endpoint already exists or absence is unproved; refusing run".into());
    }
    Ok(())
}
pub(super) struct Owner {
    pub pid: u32,
    process: OwnedHandle,
    pub log: PathBuf,
}
impl Owner {
    pub(super) fn start(manifest: &Path) -> Result<Self> {
        let startup = super::startup::launch(manifest)?;
        // Establish the expected same-connection status before constructing a
        // guard whose Drop can send Quit. Replacements are never adopted.
        let reply = ipc::call_for_process(REAL_INSTANCE, startup.pid, Command::Status)?;
        if reply.mode != Mode::Real || reply.process_id != startup.pid {
            return Err("initial status owner/mode mismatch; no owner adopted".into());
        }
        let _retained_cli = startup.cli;
        let owner = Self {
            pid: startup.pid,
            process: startup.process,
            log: startup.log,
        };
        owner.ensure()?;
        if !startup.success || reply.code != ReplyCode::Ok {
            return Err(format!(
                "real startup failed: {}; {}",
                startup.stderr,
                fs::read_to_string(&owner.log)?
            )
            .into());
        }
        Ok(owner)
    }
    pub(super) fn ensure(&self) -> Result<()> {
        if unsafe { GetProcessId(self.process.as_raw_handle()) } != self.pid
            || unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } != WAIT_TIMEOUT
        {
            return Err("real owner identity changed/exited".into());
        }
        Ok(())
    }
    pub(super) fn call(&self, command: Command) -> Result<Reply> {
        self.ensure()?;
        let reply = ipc::call_for_process(REAL_INSTANCE, self.pid, command)?;
        if reply.process_id != self.pid || reply.mode != Mode::Real {
            return Err("reply owner/mode changed".into());
        }
        println!("APP_COMMAND {command:?} {}", reply.display());
        if reply.code != ReplyCode::Ok {
            return Err(format!(
                "command failed; diagnostics {}",
                fs::read_to_string(&self.log)?
            )
            .into());
        }
        Ok(reply)
    }
    pub(super) fn wait_exit(&self) -> Result<()> {
        if unsafe { WaitForSingleObject(self.process.as_raw_handle(), 5000) } != WAIT_OBJECT_0 {
            return Err("confirmed Quit did not stop app".into());
        }
        Ok(())
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        if self.ensure().is_ok()
            && let Err(error) = self.call(Command::Quit).and_then(|_| self.wait_exit())
        {
            eprintln!(
                "APP_OWNER_CLEANUP_FAILED {error}; real owner left alive for restoration retry"
            );
        }
    }
}
