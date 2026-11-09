use super::{Command, REAL_INSTANCE, Result, ipc};
use std::{
    fs,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, Stdio},
    thread,
    time::{Duration, Instant},
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
        let cli = std::env::current_exe()?
            .parent()
            .ok_or("example directory missing")?
            .parent()
            .ok_or("debug directory missing")?
            .join("terminalhud.exe");
        let mut child = ProcessCommand::new(cli)
            .arg("--development")
            .arg(manifest)
            .arg("run")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(45);
        while child.try_wait()?.is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err("console startup timed out; inspect real owner before cleanup".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output()?;
        println!("APP_CLI {}", String::from_utf8_lossy(&output.stdout).trim());
        let stderr = String::from_utf8(output.stderr)?;
        let log = PathBuf::from(
            stderr
                .lines()
                .find_map(|line| line.strip_prefix("TerminalHUD development diagnostics: "))
                .ok_or("new real owner did not return its diagnostic path")?,
        );
        let reply = ipc::call(REAL_INSTANCE, Command::Status)?;
        if reply.mode != Mode::Real {
            return Err("owner is not a real composition".into());
        }
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                reply.process_id,
            )
        };
        if handle.is_null() {
            return Err("cannot retain real owner".into());
        }
        let owner = Self {
            pid: reply.process_id,
            process: unsafe { OwnedHandle::from_raw_handle(handle) },
            log,
        };
        owner.ensure()?;
        if !output.status.success() || reply.code != ReplyCode::Ok {
            return Err(format!(
                "real startup failed: {stderr}; {}",
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
        let reply = ipc::call(REAL_INSTANCE, command)?;
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
