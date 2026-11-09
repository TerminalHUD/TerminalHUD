use super::Result;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use terminalhud_windows_terminal::{Guid, LaunchIdentity};

pub(super) struct Setup {
    pub task: PathBuf,
    pub terminal: PathBuf,
    pub identity: LaunchIdentity,
    pub command: Vec<String>,
}
impl Setup {
    pub(super) fn load(task: &Path) -> Result<Self> {
        if !task.is_absolute() {
            return Err("explicit absolute proof directory required".into());
        }
        let record: Value = serde_json::from_slice(&fs::read(task.join("proof-record.json"))?)?;
        let identity = LaunchIdentity {
            nonce: Guid::parse(record["nonce"].as_str().ok_or("missing nonce")?)?,
            profile: Guid::parse(record["profile"].as_str().ok_or("missing profile")?)?,
        };
        let directory = task.join("portable").join("terminal-1.24.12741.0");
        if fs::read(directory.join(".portable"))? != b"" {
            return Err("portable marker differs".into());
        }
        let settings: Value =
            serde_json::from_slice(&fs::read(directory.join("settings/settings.json"))?)?;
        let fixture = task.join("fixture").join("terminal-proof.exe");
        let command = vec![
            fixture.to_string_lossy().into_owned(),
            "--shell".into(),
            task.to_string_lossy().into_owned(),
            identity.nonce.to_string(),
        ];
        let line = format!(
            "\"{}\" --shell \"{}\" {}",
            fixture.display(),
            task.display(),
            identity.nonce
        );
        validate(&settings, &identity, &line)?;
        if !fixture.is_file()
            || task.join("session-proof.json").exists()
            || task.join("exit.request").exists()
            || task.join("fixture-output-flushed.json").exists()
        {
            return Err("missing fixture or stale proof session; refusing launch".into());
        }
        Ok(Self {
            task: task.to_owned(),
            terminal: directory.join("WindowsTerminal.exe"),
            identity,
            command,
        })
    }
}

fn validate(settings: &Value, identity: &LaunchIdentity, command: &str) -> Result<()> {
    if settings["defaultProfile"] != identity.profile.as_str()
        || settings["startupActions"] != ""
        || settings["firstWindowPreference"] != "defaultProfile"
    {
        return Err("unreviewed startup settings".into());
    }
    let profiles = settings["profiles"]["list"]
        .as_array()
        .ok_or("profile list missing")?;
    if profiles.len() != 1 || profiles[0]["guid"] != identity.profile.as_str() {
        return Err("unreviewed fixture profile".into());
    }
    for profile in [&settings["profiles"]["defaults"], &profiles[0]] {
        if profile["elevate"] != false
            || profile["connectionType"] != "{00000000-0000-0000-0000-000000000000}"
            || profile["commandline"] != command
        {
            return Err("unsafe fixture/default connection, elevation or command".into());
        }
    }
    let disabled = settings["disabledProfileSources"]
        .as_array()
        .ok_or("disabled sources missing")?;
    if super::sources::DISABLED
        .iter()
        .any(|source| !disabled.iter().any(|value| value.as_str() == Some(source)))
    {
        return Err("fixture source disable list is incomplete".into());
    }
    Ok(())
}
