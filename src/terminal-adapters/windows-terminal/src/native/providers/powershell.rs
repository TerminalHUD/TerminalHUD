use super::super::NativeError;
use crate::GeneratedProfile;
use serde_json::json;
use std::path::PathBuf;

/// The common single traditional PowerShell installation. Competing Store,
/// Scoop, dotnet or alternate installations need the full upstream ranking and
/// remain unresolved; never substitute PATH's pwsh for the selected profile.
pub(super) fn discover() -> Result<Vec<GeneratedProfile<'static>>, NativeError> {
    for (variable, suffix) in [
        (
            "LOCALAPPDATA",
            "Microsoft\\WindowsApps\\Microsoft.PowerShell_8wekyb3d8bbwe",
        ),
        (
            "LOCALAPPDATA",
            "Microsoft\\WindowsApps\\Microsoft.PowerShellPreview_8wekyb3d8bbwe",
        ),
        ("USERPROFILE", ".dotnet\\tools\\pwsh.exe"),
        ("USERPROFILE", "scoop\\shims\\pwsh.exe"),
    ] {
        if std::env::var_os(variable).is_some_and(|base| PathBuf::from(base).join(suffix).exists())
        {
            return Ok(Vec::new());
        }
    }
    let mut candidates = Vec::new();
    for variable in ["ProgramFiles", "ProgramFiles(x86)", "ProgramFiles(Arm)"] {
        let Some(base) = std::env::var_os(variable) else {
            continue;
        };
        let folder = PathBuf::from(base).join("PowerShell");
        if !folder.exists() {
            continue;
        }
        for directory in std::fs::read_dir(folder)
            .map_err(|_| NativeError::Invalid("cannot read PowerShell installations"))?
        {
            let directory = directory
                .map_err(|_| NativeError::Invalid("cannot read PowerShell installation"))?;
            let executable = directory.path().join("pwsh.exe");
            if executable.is_file() && !candidates.contains(&executable) {
                candidates.push(executable);
            }
        }
    }
    if candidates.len() != 1 {
        return Ok(Vec::new());
    }
    let executable = &candidates[0];
    let folder_name = executable
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if folder_name
        .split('-')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .is_none()
    {
        return Ok(Vec::new());
    }
    let icon = if folder_name.contains("-preview") {
        "pwsh-preview.png"
    } else {
        "pwsh.png"
    };
    Ok(vec![GeneratedProfile {
        source: "Windows.Terminal.PowershellCore",
        profile: json!({
            "guid":"{574e775e-4f2a-5b96-ac1e-a2962a402336}", "name":"PowerShell",
            "commandline":format!("\"{}\"", executable.display()), "startingDirectory":"%USERPROFILE%",
            "colorScheme":"Campbell", "icon":format!("ms-appx:///ProfileIcons/{icon}")
        }),
    }])
}
