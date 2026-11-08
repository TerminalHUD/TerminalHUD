use crate::{
    protocol::Reply,
    windows::{ipc, security::Identity},
};
use std::{
    io::Read,
    process::{Command as ProcessCommand, Stdio},
    thread,
    time::{Duration, Instant},
};
use terminalhud_core::Command;

pub(super) fn ensure_running(instance: &str) -> Result<Reply, String> {
    Identity::current()
        .and_then(|id| id.pipe_name(instance))
        .map_err(|e| e.to_string())?;
    if let Ok(reply) = ipc::call(instance, Command::Status) {
        return Ok(reply);
    }
    let host = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .with_file_name("terminalhud-shell.exe");
    let mut child = ProcessCommand::new(host)
        .args(["--prototype", "--instance", instance])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if let Ok(reply) = ipc::call(instance, Command::Status) {
            return Ok(reply);
        }
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let mut error = String::new();
            if let Some(stderr) = child.stderr.take() {
                let _ = stderr.take(4096).read_to_string(&mut error);
            }
            return Err(format!(
                "GUI shell startup failed: {status}; {}",
                error.trim()
            ));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("GUI shell did not become ready".into());
        }
        thread::sleep(Duration::from_millis(25));
    }
}
