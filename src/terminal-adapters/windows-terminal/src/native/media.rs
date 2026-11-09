use super::{NativeError, command::wide, error::last};
use serde_json::{Map, Value};
use std::path::Path;
use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsW;

/// Read a fragment and retain the meaning of known media properties when moving
/// its profile to user-settings precedence. Never changes the fragment itself.
pub fn read_profile_fragment(path: &Path, namespace: &str) -> Result<String, NativeError> {
    if !path.is_absolute() {
        return Err(NativeError::Invalid("absolute fragment path required"));
    }
    let text = std::fs::read_to_string(path)
        .map_err(|_| NativeError::Invalid("cannot read Terminal fragment"))?;
    let mut root = crate::profiles::parse::document(&text).map_err(NativeError::Correlation)?;
    if let Some(Value::Array(profiles)) = root.get_mut("profiles") {
        for profile in profiles {
            let object = profile
                .as_object_mut()
                .ok_or(NativeError::Invalid("invalid fragment profile"))?;
            if !object.contains_key("guid")
                && !object.contains_key("updates")
                && let Some(name) = object.get("name").and_then(Value::as_str)
            {
                let guid = super::providers::fragment_guid(namespace, name)?;
                object.insert("guid".into(), Value::String(guid.to_string()));
            }
            resolve(object, path.parent().unwrap())?;
            if let Some(Value::Object(unfocused)) = object.get_mut("unfocusedAppearance") {
                resolve(unfocused, path.parent().unwrap())?;
            }
        }
    }
    serde_json::to_string(&root).map_err(|_| NativeError::Invalid("cannot serialize fragment"))
}
fn resolve(values: &mut Map<String, Value>, base: &Path) -> Result<(), NativeError> {
    for key in [
        "icon",
        "backgroundImage",
        "experimental.pixelShaderPath",
        "experimental.pixelShaderImagePath",
    ] {
        if let Some(Value::String(value)) = values.get_mut(key) {
            *value = media(value, base, key == "icon")?;
        }
    }
    if let Some(Value::String(sound)) = values.get_mut("bellSound") {
        *sound = media(sound, base, false)?;
    }
    if let Some(Value::Array(sounds)) = values.get_mut("bellSound") {
        for sound in sounds {
            if let Value::String(value) = sound {
                *value = media(value, base, false)?;
            }
        }
    }
    Ok(())
}
fn media(raw: &str, base: &Path, icon: bool) -> Result<String, NativeError> {
    if raw.is_empty()
        || raw.eq_ignore_ascii_case("none")
        || raw.eq_ignore_ascii_case("desktopWallpaper")
        || (icon && raw.chars().count() <= 2 && !raw.is_ascii())
    {
        return Ok(raw.into());
    }
    let wide = wide(raw.as_ref())?;
    let required = unsafe { ExpandEnvironmentStringsW(wide.as_ptr(), std::ptr::null_mut(), 0) };
    if required == 0 {
        return Err(last("ExpandEnvironmentStringsW fragment"));
    }
    if required > 32768 {
        return Err(NativeError::Invalid("fragment media path too long"));
    }
    let mut expanded = vec![0u16; required as usize];
    let written =
        unsafe { ExpandEnvironmentStringsW(wide.as_ptr(), expanded.as_mut_ptr(), required) };
    if written == 0 || written > required {
        return Err(NativeError::Invalid("fragment media environment changed"));
    }
    let mut value = String::from_utf16(&expanded[..written as usize - 1])
        .map_err(|_| NativeError::Invalid("invalid fragment media UTF-16"))?;
    let lower = value.to_ascii_lowercase();
    // Terminal converts remote fragment media URLs to a local basename, not a
    // network download. Hosted ms-appx URLs follow the same rule.
    if lower.starts_with("http:")
        || lower.starts_with("https:")
        || (lower.starts_with("ms-appx://") && !lower.starts_with("ms-appx:///"))
    {
        let path = value.split(['?', '#']).next().unwrap_or("");
        value = path.rsplit('/').next().unwrap_or("").to_owned();
        if value.contains('%') {
            return Err(NativeError::Unsupported(
                "escaped fragment URI media requires resolution",
            ));
        }
    } else if lower.starts_with("file:") {
        value = file_uri_path(&value)?;
    } else if lower.starts_with("ms-") {
        return Ok(value);
    }
    // A fragment's disallowed UNC resource must not become usable by acquiring
    // user-profile origin. Preserve the rejected/empty resource.
    if value.starts_with("\\\\") || value.starts_with("//") {
        return Ok(String::new());
    }
    if value.is_empty() {
        return Ok(value);
    }
    let (path, index) = if icon {
        value
            .rsplit_once(',')
            .filter(|(p, n)| {
                n.parse::<i32>().is_ok()
                    && [".exe", ".dll", ".lnk"]
                        .iter()
                        .any(|suffix| p.to_ascii_lowercase().ends_with(suffix))
            })
            .map(|(p, n)| (p, format!(",{n}")))
            .unwrap_or((&value, String::new()))
    } else {
        (&value[..], String::new())
    };
    let resolved = base.join(path);
    // Missing files are rejected by Terminal too, but keep the absolute path so
    // a later appearance reload can resolve a file created after preparation.
    Ok(format!("{}{index}", resolved.display()))
}
// Only local drive file URIs have an unambiguous supported conversion here.
// Decode before checking UNC so acquiring User origin cannot enable a resource
// that Terminal rejected at Fragment origin. No filesystem/network reads occur.
fn file_uri_path(uri: &str) -> Result<String, NativeError> {
    if !uri.to_ascii_lowercase().starts_with("file:///") {
        return Err(NativeError::Unsupported("unsupported fragment file URI"));
    }
    // Uri.Path excludes raw query/fragment components; encoded delimiters are
    // part of the filename and are decoded only after this split.
    let path_component = uri.split(['?', '#']).next().unwrap_or(uri);
    let bytes = path_component.as_bytes();
    let mut decoded = Vec::new();
    let mut index = 8;
    while index < bytes.len() {
        let byte = if bytes[index] == b'%' {
            let hex = bytes
                .get(index + 1..index + 3)
                .and_then(|h| std::str::from_utf8(h).ok());
            let value = hex
                .and_then(|h| u8::from_str_radix(h, 16).ok())
                .ok_or(NativeError::Invalid("invalid file URI escape"))?;
            index += 3;
            value
        } else {
            let value = bytes[index];
            index += 1;
            value
        };
        if byte == 0 {
            return Err(NativeError::Invalid("NUL in file URI"));
        }
        decoded.push(byte);
    }
    let path =
        String::from_utf8(decoded).map_err(|_| NativeError::Invalid("invalid file URI UTF-8"))?;
    if path.starts_with("\\\\") || path.starts_with("//") {
        return Ok(String::new());
    }
    let bytes = path.as_bytes();
    if bytes.len() < 3
        || !bytes[0].is_ascii_alphabetic()
        || bytes[1] != b':'
        || !matches!(bytes[2], b'/' | b'\\')
    {
        return Err(NativeError::Unsupported(
            "fragment file URI is not a local drive path",
        ));
    }
    Ok(path)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_uri_and_scalar_sound_keep_fragment_restrictions_and_origin() {
        let base = Path::new(r"C:\fixtures\vendor");
        assert_eq!(
            media("file:///%5C%5Cserver%5Cshare%5Cwall.png", base, false).unwrap(),
            ""
        );
        assert_eq!(
            media("file:///C:/images/my%20wall.png", base, false).unwrap(),
            "C:/images/my wall.png"
        );
        assert_eq!(
            media("file:///C:/images/wall.png#preview", base, false).unwrap(),
            "C:/images/wall.png"
        );
        assert_eq!(
            media("file:///C:/images/wall.png?query", base, false).unwrap(),
            "C:/images/wall.png"
        );
        assert_eq!(
            media("file:///C:/images/wall%23preview.png", base, false).unwrap(),
            "C:/images/wall#preview.png"
        );
        assert!(media("file://server/share/wall.png", base, false).is_err());
        let mut values = serde_json::json!({"bellSound":"bell.wav"})
            .as_object()
            .unwrap()
            .clone();
        resolve(&mut values, base).unwrap();
        assert_eq!(values["bellSound"], r"C:\fixtures\vendor\bell.wav");
    }
    #[test]
    fn media_keeps_fragment_origin_without_enabling_unc_resources() {
        let base = Path::new(r"C:\fixtures\vendor");
        assert_eq!(
            media("wall.png", base, false).unwrap(),
            r"C:\fixtures\vendor\wall.png"
        );
        assert_eq!(
            media("https://example.test/path/icon.png", base, true).unwrap(),
            r"C:\fixtures\vendor\icon.png"
        );
        assert_eq!(media(r"\\server\share\wall.png", base, false).unwrap(), "");
        assert_eq!(
            media("ms-appx:///icons/cmd.png", base, true).unwrap(),
            "ms-appx:///icons/cmd.png"
        );
        assert_eq!(
            media("desktopWallpaper", base, false).unwrap(),
            "desktopWallpaper"
        );
    }
}
