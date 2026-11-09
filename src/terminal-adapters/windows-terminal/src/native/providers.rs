//! Bounded native provider discovery. No shell execution or registry writes.
use super::{NativeError, command::wide};
use crate::{GeneratedProfile, Guid};
use serde_json::json;
use std::{
    path::PathBuf,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
    Security::Cryptography::*,
    System::{Registry::*, SystemInformation::GetSystemDirectoryW},
};

struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}
fn open(parent: HKEY, name: &str) -> Result<Option<Key>, NativeError> {
    let name = wide(name.as_ref())?;
    let mut key = null_mut();
    let status = unsafe { RegOpenKeyExW(parent, name.as_ptr(), 0, KEY_READ, &mut key) };
    match status {
        ERROR_SUCCESS => Ok(Some(Key(key))),
        ERROR_FILE_NOT_FOUND => Ok(None),
        _ => Err(NativeError::Win32("RegOpenKeyExW provider", status)),
    }
}
fn string(key: HKEY, name: &str) -> Result<String, NativeError> {
    let name = wide(name.as_ref())?;
    let mut bytes = 0;
    let status = unsafe {
        RegGetValueW(
            key,
            null(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            null_mut(),
            &mut bytes,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(NativeError::Win32("RegGetValueW provider size", status));
    }
    if !(2..=65536).contains(&bytes) || bytes % 2 != 0 {
        return Err(NativeError::Invalid("invalid provider string size"));
    }
    let mut value = vec![0u16; bytes as usize / 2];
    let status = unsafe {
        RegGetValueW(
            key,
            null(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            value.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(NativeError::Win32("RegGetValueW provider", status));
    }
    let len = bytes as usize / 2;
    if len == 0 || len > value.len() || value[len - 1] != 0 || value[..len - 1].contains(&0) {
        return Err(NativeError::Invalid("invalid provider string"));
    }
    String::from_utf16(&value[..len - 1])
        .map_err(|_| NativeError::Invalid("invalid provider UTF-16"))
}
/// Windows 11 legacy WSL registry generator; modern WSL distributions provide
/// their own fragments and are intentionally not generated a second time.
pub fn discover_generated_profiles() -> Result<Vec<GeneratedProfile<'static>>, NativeError> {
    let mut profiles = powershell()?;
    let Some(root) = open(
        HKEY_CURRENT_USER,
        "Software\\Microsoft\\Windows\\CurrentVersion\\Lxss",
    )?
    else {
        return Ok(profiles);
    };
    let mut system = vec![0u16; 32768];
    let length = unsafe { GetSystemDirectoryW(system.as_mut_ptr(), system.len() as u32) } as usize;
    if length == 0 || length >= system.len() {
        return Err(NativeError::Invalid("system directory unavailable"));
    }
    let system = String::from_utf16(&system[..length])
        .map_err(|_| NativeError::Invalid("invalid system directory"))?;
    let wsl = PathBuf::from(system).join("wsl.exe");
    if !wsl.is_file() {
        return Ok(profiles);
    }
    for index in 0..4096 {
        let mut name = [0u16; 256];
        let mut length = name.len() as u32;
        let status = unsafe {
            RegEnumKeyExW(
                root.0,
                index,
                name.as_mut_ptr(),
                &mut length,
                null(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };
        if status == ERROR_NO_MORE_ITEMS {
            return Ok(profiles);
        }
        if status != ERROR_SUCCESS {
            return Err(NativeError::Win32("RegEnumKeyExW WSL", status));
        }
        let key_name = String::from_utf16(&name[..length as usize])
            .map_err(|_| NativeError::Invalid("invalid WSL key"))?;
        if Guid::parse(&key_name).is_err() {
            continue;
        }
        let Some(key) = open(root.0, &key_name)? else {
            continue;
        };
        let mut modern = 0u32;
        let mut bytes = 4;
        let modern_name = wide("Modern".as_ref())?;
        let status = unsafe {
            RegGetValueW(
                key.0,
                null(),
                modern_name.as_ptr(),
                RRF_RT_REG_DWORD,
                null_mut(),
                (&mut modern as *mut u32).cast(),
                &mut bytes,
            )
        };
        if status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND {
            return Err(NativeError::Win32("RegGetValueW WSL Modern", status));
        }
        if modern == 1 {
            continue;
        }
        let name = string(key.0, "DistributionName")?;
        if name.starts_with("docker-desktop") || name.starts_with("rancher-desktop") {
            continue;
        }
        // The upstream generator appends an unquoted distro name. Refuse names
        // requiring quoting rather than launching a different command grammar.
        if name.is_empty()
            || name
                .chars()
                .any(|c| c.is_whitespace() || matches!(c, '"' | ';'))
        {
            continue;
        }
        let guid = dynamic_guid(&name)?;
        profiles.push(GeneratedProfile { source: "Windows.Terminal.Wsl", profile: json!({
            "guid":guid.as_str(), "name":name, "commandline":format!("{} -d {}", wsl.display(), name),
            "startingDirectory":"~", "colorScheme":"Campbell", "pathTranslationStyle":"wsl",
            "icon":"ms-appx:///ProfileIcons/{9acb9455-ca41-5af7-950f-6bca1bc9722f}.png"
        }) });
    }
    Err(NativeError::Invalid("too many WSL registrations"))
}

/// The common single traditional PowerShell installation. Competing Store,
/// Scoop, dotnet or alternate installations need the full upstream ranking and
/// remain unresolved; never substitute PATH's pwsh for the selected profile.
fn powershell() -> Result<Vec<GeneratedProfile<'static>>, NativeError> {
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

fn dynamic_guid(name: &str) -> Result<Guid, NativeError> {
    guid_from_bytes(uuid5(
        [
            0x2b, 0xde, 0x4a, 0x90, 0xd0, 0x5f, 0x40, 0x1c, 0x94, 0x92, 0xe4, 0x08, 0x84, 0xea,
            0xd1, 0xd8,
        ],
        name,
    )?)
}
pub(super) fn fragment_guid(namespace: &str, name: &str) -> Result<Guid, NativeError> {
    let namespace = uuid5(
        [
            0xf6, 0x5d, 0xdb, 0x7e, 0x70, 0x6b, 0x44, 0x99, 0x8a, 0x50, 0x40, 0x31, 0x3c, 0xaf,
            0x51, 0x0a,
        ],
        namespace,
    )?;
    guid_from_bytes(uuid5(namespace, name)?)
}
fn uuid5(namespace: [u8; 16], name: &str) -> Result<[u8; 16], NativeError> {
    let mut data = namespace.to_vec();
    for unit in name.encode_utf16() {
        data.extend(unit.to_le_bytes());
    }
    let mut algorithm = null_mut();
    let status =
        unsafe { BCryptOpenAlgorithmProvider(&mut algorithm, BCRYPT_SHA1_ALGORITHM, null(), 0) };
    if status < 0 {
        return Err(NativeError::Invalid("SHA1 provider unavailable"));
    }
    let mut hash = [0u8; 20];
    let status = unsafe {
        BCryptHash(
            algorithm,
            null(),
            0,
            data.as_ptr(),
            data.len() as u32,
            hash.as_mut_ptr(),
            hash.len() as u32,
        )
    };
    unsafe {
        BCryptCloseAlgorithmProvider(algorithm, 0);
    }
    if status < 0 {
        return Err(NativeError::Invalid("profile UUID hashing failed"));
    }
    hash[6] = (hash[6] & 0x0f) | 0x50;
    hash[8] = (hash[8] & 0x3f) | 0x80;
    Ok(hash[..16].try_into().expect("UUID length"))
}
fn guid_from_bytes(hash: [u8; 16]) -> Result<Guid, NativeError> {
    let hex: String = hash[..16].iter().map(|b| format!("{b:02x}")).collect();
    Guid::parse(&format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
    .map_err(NativeError::Correlation)
}
#[cfg(test)]
mod tests {
    #[test]
    fn upstream_wsl_identity_uses_utf16_uuid_namespace() {
        assert_eq!(
            super::dynamic_guid("Ubuntu").unwrap().as_str(),
            "{2c4de342-38b7-51cf-b940-2309a097f518}"
        );
        assert_eq!(
            super::dynamic_guid("Debian").unwrap().as_str(),
            "{58ad8b0c-3ef8-5f4d-bc6f-13e4c00f2530}"
        );
    }
}
