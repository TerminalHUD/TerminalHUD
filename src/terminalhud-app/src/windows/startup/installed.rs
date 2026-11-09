use super::StartupError;
use super::paths::{regular_path, safe_build};
use super::receipt::Receipt;
use std::{fs, io::Read, path::PathBuf};

#[derive(Debug)]
pub(super) struct Installed {
    executable: PathBuf,
    pub command: String,
    pub owned_commands: Vec<String>,
}

impl Installed {
    pub fn current() -> Result<Self, StartupError> {
        Self::load(std::env::current_exe().map_err(|e| e.to_string())?)
    }
    pub fn refresh(&self) -> Result<Self, StartupError> {
        Self::load(self.executable.clone())
    }
    fn load(executable: PathBuf) -> Result<Self, StartupError> {
        let settings = crate::config::settings_path()?;
        let root = settings
            .parent()
            .ok_or("settings parent missing".to_string())?
            .join("install");
        Self::load_at(executable, root)
    }
    fn load_at(executable: PathBuf, root: PathBuf) -> Result<Self, StartupError> {
        let relative = executable
            .strip_prefix(root.join("versions"))
            .map_err(|_| "startup requires a receipted per-user installation".to_string())?;
        let pieces: Vec<_> = relative.components().collect();
        if pieces.len() != 2 {
            return Err("invalid installed executable location".to_string().into());
        }
        let build = pieces[0]
            .as_os_str()
            .to_str()
            .ok_or("non-Unicode build identity".to_string())?;
        if !safe_build(build) {
            return Err("unsafe installed build identity".to_string().into());
        }
        let name = pieces[1].as_os_str();
        if name != "terminalhud.exe" && name != "terminalhud-shell.exe" {
            return Err("unexpected installed executable".to_string().into());
        }
        let gui = root
            .join("versions")
            .join(build)
            .join("terminalhud-shell.exe");
        let receipt_path = root.join("receipt.json");
        for path in [&executable, &gui, &receipt_path] {
            regular_path(path)?;
        }
        let mut bytes = Vec::new();
        fs::File::open(receipt_path)
            .map_err(|e| e.to_string())?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 1024 * 1024 {
            return Err("installation receipt exceeds 1 MiB".to_string().into());
        }
        let receipt: Receipt = serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid installation receipt: {e}"))?;
        if !receipt.owns_gui(build) {
            return Err("installation receipt does not own active GUI executable"
                .to_string()
                .into());
        }
        let command = format!(
            "\"{}\"",
            gui.to_str()
                .ok_or("non-Unicode installed GUI path".to_string())?
        );
        let mut owned_commands = Vec::new();
        for version in &receipt.versions {
            if !safe_build(&version.build_id) {
                return Err("unsafe receipted build identity".to_string().into());
            }
            let candidate = format!(
                "\"{}\"",
                root.join("versions")
                    .join(&version.build_id)
                    .join("terminalhud-shell.exe")
                    .display()
            );
            if receipt.startup_commands.contains(&candidate) {
                owned_commands.push(candidate);
            }
        }
        if !owned_commands.contains(&command) {
            return Err("installation receipt does not own startup command"
                .to_string()
                .into());
        }
        Ok(Self {
            executable,
            command,
            owned_commands,
        })
    }
}

#[cfg(test)]
#[path = "installed_tests.rs"]
mod tests;
