//! Fail closed on unreviewed fragment/app-extension namespaces.
use super::error;
use std::{
    io::{self, Read},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
pub(super) const DISABLED: &[&str] = &[
    "Windows.Terminal.PowershellCore",
    "Windows.Terminal.Wsl",
    "Windows.Terminal.Azure",
    "Windows.Terminal.VisualStudio",
    "Windows.Terminal.SSH",
    "Microsoft.WSL",
];
pub(super) fn powershell() -> io::Result<std::path::PathBuf> {
    Ok(std::path::PathBuf::from(
        std::env::var_os("SystemRoot").ok_or_else(|| error("SystemRoot missing"))?,
    )
    .join("System32/WindowsPowerShell/v1.0/powershell.exe"))
}
pub(super) fn check() -> io::Result<()> {
    let mut child = Command::new(powershell()?)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            include_str!("namespaces.ps1"),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(4);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error(
                "Terminal namespace discovery timed out; no target launched",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    };
    if !status.success() {
        return Err(error(
            "Terminal namespace discovery failed; refusing launch",
        ));
    }
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or_else(|| error("discovery stdout missing"))?
        .take(65537)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(error("namespace discovery exceeded bound"));
    }
    let sources: Vec<String> = serde_json::from_slice(&bytes).map_err(error)?;
    if sources
        .iter()
        .any(|source| !DISABLED.contains(&source.as_str()))
    {
        return Err(error(
            "unreviewed Terminal fragment namespace; refusing launch",
        ));
    }
    Ok(())
}
