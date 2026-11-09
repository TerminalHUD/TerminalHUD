use super::{Command, REAL_INSTANCE, Result, ipc};
use std::{
    cell::Cell,
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
    pub startup_failure: Option<String>,
    pub startup_reply: Reply,
    quit_dispatched: Cell<bool>,
    quit_confirmed: Cell<bool>,
}
impl Owner {
    pub(super) fn start(manifest: &Path) -> Result<Self> {
        let owner = Self::start_observed(manifest)?;
        if let Some(error) = &owner.startup_failure {
            return Err(error.clone().into());
        }
        Ok(owner)
    }
    pub(super) fn start_observed(manifest: &Path) -> Result<Self> {
        let startup = super::startup::launch(manifest)?;
        // Establish the expected same-connection status before constructing a
        // guard whose Drop can send Quit. Replacements are never adopted.
        let reply = ipc::call_for_process(REAL_INSTANCE, startup.pid, Command::Status)?;
        if reply.mode != Mode::Real || reply.process_id != startup.pid {
            return Err("initial status owner/mode mismatch; no owner adopted".into());
        }
        let _retained_cli = startup.cli;
        let mut owner = Self {
            pid: startup.pid,
            process: startup.process,
            log: startup.log,
            startup_reply: reply,
            startup_failure: None,
            quit_dispatched: Cell::new(false),
            quit_confirmed: Cell::new(false),
        };
        owner.ensure()?;
        if !startup.success || reply.code != ReplyCode::Ok {
            owner.startup_failure = Some(format!(
                "real startup failed: {}; {}",
                startup.stderr,
                fs::read_to_string(&owner.log)?
            ));
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
        if command == Command::Quit && self.quit_dispatched.replace(true) {
            return Err("Quit already attempted; no retry".into());
        }
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
        if command == Command::Quit
            && reply.status.availability != terminalhud_core::integration::Availability::Unmanaged
        {
            return Err("Quit did not confirm Unmanaged; restoration remains unconfirmed".into());
        }
        if command == Command::Quit {
            self.quit_confirmed.set(true);
        }
        Ok(reply)
    }
    pub(super) fn epoch(&self) -> Result<serde_json::Value> {
        super::startup_identity::process_json(self.process.as_raw_handle())
    }
    pub(super) fn quit_state(&self) -> (bool, bool, bool) {
        (
            self.quit_dispatched.get(),
            self.quit_confirmed.get(),
            unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } == WAIT_OBJECT_0,
        )
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
        if !self.quit_dispatched.get()
            && self.ensure().is_ok()
            && let Err(error) = self.call(Command::Quit).and_then(|_| self.wait_exit())
        {
            eprintln!(
                "APP_OWNER_CLEANUP_FAILED {error}; real owner left alive with restoration pending; no automatic retry"
            );
        }
    }
}
