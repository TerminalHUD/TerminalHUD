use super::{error, namespaces};
use serde_json::Value;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use terminalhud_windows_terminal::Guid;
pub(in crate::windows) struct Configuration {
    pub terminal: PathBuf,
    pub profile: Guid,
    pub bridge: PathBuf,
}
impl Configuration {
    pub(in crate::windows) fn load(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() {
            return Err(error("absolute development manifest required"));
        }
        let value: Value = serde_json::from_slice(&fs::read(path)?).map_err(error)?;
        let terminal = PathBuf::from(
            value["terminal"]
                .as_str()
                .ok_or_else(|| error("manifest terminal missing"))?,
        );
        let profile = Guid::parse(
            value["profile"]
                .as_str()
                .ok_or_else(|| error("manifest profile missing"))?,
        )
        .map_err(error)?;
        let bridge = std::env::current_exe()?.with_file_name("terminalhud.exe");
        let config = Self {
            terminal,
            profile,
            bridge,
        };
        config.validate()?;
        Ok(config)
    }
    pub(super) fn fallback(&self) -> String {
        format!("\"{}\" --session-bridge", self.bridge.display())
    }
    pub(super) fn validate(&self) -> io::Result<()> {
        if !self.terminal.is_absolute() || !self.terminal.is_file() || !self.bridge.is_file() {
            return Err(error(
                "absolute portable Terminal and sibling console bridge required",
            ));
        }
        let directory = self
            .terminal
            .parent()
            .ok_or_else(|| error("portable directory missing"))?;
        if fs::read(directory.join(".portable"))? != b"" {
            return Err(error("portable marker differs"));
        }
        let settings: Value =
            serde_json::from_slice(&fs::read(directory.join("settings/settings.json"))?)
                .map_err(error)?;
        validate_settings(&settings, &self.profile, &self.fallback())
    }
}
fn validate_settings(settings: &Value, profile: &Guid, fallback: &str) -> io::Result<()> {
    if settings["defaultProfile"] != profile.as_str()
        || !(settings["startupActions"].is_null() || settings["startupActions"] == "")
        || settings["firstWindowPreference"] != "defaultProfile"
    {
        return Err(error("unsafe development startup settings"));
    }
    let hud = super::runtime_profiles::hud_profile(settings, profile)?;
    for (defaults, item) in [(true, &settings["profiles"]["defaults"]), (false, hud)] {
        if item["elevate"] != false
            || item["connectionType"] != "{00000000-0000-0000-0000-000000000000}"
            || !(item["commandline"] == fallback || (defaults && item.get("commandline").is_none()))
            || item["opacity"] != 35
            || item["useAcrylic"] != false
            || item["suppressApplicationTitle"] != true
            || item["unfocusedAppearance"]["opacity"] != 0
            || item["unfocusedAppearance"]["useAcrylic"] != false
        {
            return Err(error(
                "unsafe development profile/fallback or unreviewed appearance",
            ));
        }
    }
    let disabled = settings["disabledProfileSources"]
        .as_array()
        .ok_or_else(|| error("disabled namespaces missing"))?;
    if namespaces::DISABLED
        .iter()
        .any(|source| !disabled.iter().any(|value| value.as_str() == Some(source)))
    {
        return Err(error("development disabled namespace list incomplete"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
