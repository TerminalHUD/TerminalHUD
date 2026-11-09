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

pub(super) fn ensure_running(args: &super::Arguments) -> Result<Reply, String> {
    let instance = &args.instance;
    Identity::current()
        .and_then(|id| id.pipe_name(instance))
        .map_err(|e| e.to_string())?;
    if let Ok(reply) = ipc::call(instance, Command::Status) {
        return Ok(reply);
    }
    let host = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .with_file_name("terminalhud-shell.exe");
    let mut command = ProcessCommand::new(host);
    if args.mode == crate::mode::Mode::Prototype {
        command.args(["--prototype", "--instance", instance]);
    } else {
        command
            .arg("--development")
            .arg(args.development.as_ref().ok_or("missing manifest")?);
    }
    let diagnostic = if args.mode == crate::mode::Mode::Real {
        let name = format!(
            "TerminalHUD-development-start-{}-{}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos()
        );
        Some(
            crate::windows::development::temporary_directory()
                .map_err(|e| e.to_string())?
                .join(name),
        )
    } else {
        None
    };
    let stderr = match &diagnostic {
        Some(path) => Stdio::from(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|e| e.to_string())?,
        ),
        None => Stdio::piped(),
    };
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr)
        .spawn()
        .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Ok(reply) = ipc::call(instance, Command::Status) {
            if let Some(path) = &diagnostic {
                eprintln!("TerminalHUD development diagnostics: {}", path.display());
            }
            return Ok(reply);
        }
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let mut error = String::new();
            if let Some(stderr) = child.stderr.take() {
                let _ = stderr.take(4096).read_to_string(&mut error);
            }
            if let Some(path) = &diagnostic {
                error = std::fs::read_to_string(path)
                    .unwrap_or_default()
                    .chars()
                    .take(4096)
                    .collect();
            }
            return Err(format!(
                "GUI shell startup failed: {status}; {}",
                error.trim()
            ));
        }
        if Instant::now() >= deadline {
            if args.mode == crate::mode::Mode::Prototype {
                let _ = child.kill();
                let _ = child.wait();
            }
            return Err(format!(
                "GUI shell did not become ready; real owner was left alive to preserve restoration responsibility; inspect `terminalhud status` before retrying run; diagnostics={diagnostic:?}"
            ));
        }
        thread::sleep(Duration::from_millis(25));
    }
}
