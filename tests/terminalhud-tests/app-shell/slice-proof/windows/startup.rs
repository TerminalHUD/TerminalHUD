//! Adopt only the exact GUI child born during our retained CLI's lifetime.
use super::Result;
use std::{
    os::windows::io::OwnedHandle,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};
use terminalhud_app::windows::process_output::Capture;
pub(super) struct Startup {
    pub pid: u32,
    pub process: OwnedHandle,
    pub log: PathBuf,
    pub success: bool,
    pub stderr: String,
    pub cli: std::process::Child,
}
pub(super) fn launch(manifest: &Path) -> Result<Startup> {
    let cli = std::env::current_exe()?
        .parent()
        .ok_or("example directory missing")?
        .parent()
        .ok_or("debug directory missing")?
        .join("terminalhud.exe");
    let capture = Capture::new()?;
    println!("APP_CAPTURE directory={}", capture.directory.display());
    let mut command = Command::new(&cli);
    command.arg("--development").arg(manifest).arg("run");
    capture.redirect(&mut command)?;
    let mut child = command.spawn()?;
    let deadline = Instant::now() + Duration::from_secs(45);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "startup CLI {} timed out; no owner adopted and no cleanup command sent",
                child.id()
            )
            .into());
        }
        thread::sleep(Duration::from_millis(10));
    };
    let (stdout, stderr) = capture.read()?;
    let stdout = String::from_utf8(stdout)?;
    let stderr = String::from_utf8(stderr)?;
    println!("APP_CLI {}", stdout.trim());
    let response = stdout
        .lines()
        .chain(stderr.lines())
        .find(|line| line.contains("mode=real ") && line.contains(" pid="))
        .ok_or("own CLI returned no real owner identity; no owner adopted")?;
    let pid = response
        .split_whitespace()
        .find_map(|word| word.strip_prefix("pid="))
        .ok_or("own CLI PID missing")?
        .parse::<u32>()?;
    let process = super::startup_identity::retain(pid, &child, &cli)?;
    let log = PathBuf::from(
        stderr
            .lines()
            .find_map(|line| line.strip_prefix("TerminalHUD development diagnostics: "))
            .ok_or("own CLI returned no diagnostic path")?,
    );
    Ok(Startup {
        pid,
        process,
        log,
        success: status.success(),
        stderr,
        cli: child,
    })
}
