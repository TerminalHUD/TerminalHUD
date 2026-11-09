//! Read installed Terminal inputs without changing its settings or catalog.
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use terminalhud_windows_terminal::{TerminalChannel, discover_registered_terminals};

pub(super) struct Sources {
    pub executable: PathBuf,
    pub settings: PathBuf,
    pub local_app_data: PathBuf,
    pub fragments: Vec<(String, String)>,
}
impl Sources {
    pub fn discover() -> io::Result<Self> {
        let discovery = discover_registered_terminals();
        let local = discovery
            .local_app_data
            .map_err(|e| io::Error::other(format!("Local App Data unavailable: {e:?}")))?;
        let mut selected = None;
        for channel in [TerminalChannel::Stable, TerminalChannel::Preview] {
            let entry = discovery
                .channels
                .iter()
                .find(|entry| entry.channel == channel)
                .expect("known channel");
            let candidates = entry.candidates.as_ref().map_err(|e| {
                io::Error::other(format!("Terminal package discovery failed: {e:?}"))
            })?;
            if candidates.len() > 1 {
                return Err(io::Error::other(
                    "multiple Windows Terminal registrations require selection",
                ));
            }
            if let Some(candidate) = candidates.first() {
                selected = Some(candidate);
                break;
            }
        }
        let candidate = selected.ok_or_else(|| {
            io::Error::other("Install Windows Terminal before starting TerminalHUD")
        })?;
        let executable = candidate
            .expected_executable
            .clone()
            .map_err(|e| io::Error::other(format!("Terminal executable unavailable: {e:?}")))?;
        let settings = candidate
            .expected_settings
            .clone()
            .map_err(|e| io::Error::other(format!("Terminal settings unavailable: {e:?}")))?;
        if !executable.is_file() {
            return Err(io::Error::other(
                "registered Windows Terminal executable is unavailable",
            ));
        }
        let mut fragments = Vec::new();
        collect(
            &local.join("Microsoft/Windows Terminal/Fragments"),
            &mut fragments,
        )?;
        if let Some(program_data) = std::env::var_os("ProgramData") {
            collect(
                &PathBuf::from(program_data).join("Microsoft/Windows Terminal/Fragments"),
                &mut fragments,
            )?;
        }
        Ok(Self {
            executable,
            settings,
            local_app_data: local,
            fragments,
        })
    }
}
pub(super) fn read(path: &Path) -> io::Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > 1024 * 1024 {
        return Err(io::Error::other(
            "Terminal settings input is not a bounded regular file",
        ));
    }
    fs::read_to_string(path)
}
fn collect(root: &Path, output: &mut Vec<(String, String)>) -> io::Result<()> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for namespace in entries {
        let namespace = namespace?;
        if !namespace.file_type()?.is_dir() {
            continue;
        }
        let name = namespace
            .file_name()
            .into_string()
            .map_err(|_| io::Error::other("invalid fragment namespace"))?;
        for file in fs::read_dir(namespace.path())? {
            let file = file?;
            if file.file_type()?.is_file()
                && file
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                if output.len() >= 256 {
                    return Err(io::Error::other("too many Terminal fragments"));
                }
                let _ = read(&file.path())?;
                let text = terminalhud_windows_terminal::native::read_profile_fragment(
                    &file.path(),
                    &name,
                )
                .map_err(io::Error::other)?;
                output.push((name.clone(), text));
            }
        }
    }
    Ok(())
}
