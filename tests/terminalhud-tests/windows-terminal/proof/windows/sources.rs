//! The controlled portable fixture disables every observed external namespace.
use super::Result;
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

pub(super) const DISABLED: &[&str] = &[
    "Windows.Terminal.PowershellCore",
    "Windows.Terminal.Wsl",
    "Windows.Terminal.Azure",
    "Windows.Terminal.VisualStudio",
    "Windows.Terminal.SSH",
    "Microsoft.WSL",
];

pub(super) fn check() -> Result<Vec<String>> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            include_str!("sources.ps1"),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "namespace discovery failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let sources: Vec<String> = serde_json::from_slice(&output.stdout)?;
    if let Some(source) = sources
        .iter()
        .find(|source| !DISABLED.contains(&source.as_str()))
    {
        return Err(format!("unreviewed Terminal namespace: {source}; refusing launch").into());
    }
    Ok(sources)
}
