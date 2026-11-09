//! The ConPTY client stays independent of the tray owner's lifetime.
use super::{error, namespaces};
use std::{fs::OpenOptions, io::Write, path::Path, process::Command};
use terminalhud_windows_terminal::Guid;
pub fn run_bridge(args: &[String]) -> Result<(), String> {
    run(args).map_err(|e| e.to_string())
}
fn run(args: &[String]) -> std::io::Result<()> {
    if args.len() != 3 {
        return Err(error(
            "bridge requires an owned session directory, nonce and profile; fallback refused",
        ));
    }
    let directory = Path::new(&args[0]);
    let nonce = Guid::parse(&args[1]).map_err(error)?;
    let profile = Guid::parse(&args[2]).map_err(error)?;
    if !directory.is_absolute()
        || std::fs::read(directory.join("nonce"))? != nonce.as_str().as_bytes()
    {
        return Err(error("session nonce mismatch"));
    }
    let actual_profile =
        Guid::parse(&std::env::var("WT_PROFILE_ID").map_err(error)?).map_err(error)?;
    let session = Guid::parse(&std::env::var("WT_SESSION").map_err(error)?).map_err(error)?;
    if profile != actual_profile {
        return Err(error("Terminal did not load requested development profile"));
    }
    super::super::recovery::start(nonce.bare().replace('-', ""))?;
    let mut child = Command::new(namespaces::powershell()?)
        .args(["-NoLogo", "-NoProfile"])
        .spawn()?;
    let proof = serde_json::json!({"nonce": nonce.as_str(), "profile": profile.as_str(), "session": session.as_str(), "shellPid": std::process::id()});
    // Atomic publication; never overwrite another session's handshake.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("session.tmp"))?;
    file.write_all(serde_json::to_string(&proof).map_err(error)?.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(
        directory.join("session.tmp"),
        directory.join("session.json"),
    )?;
    let status = child.wait()?;
    if !status.success() {
        return Err(error(format!("interactive PowerShell exited: {status}")));
    }
    Ok(())
}
