use super::StartupError;
use std::{fs, os::windows::fs::MetadataExt, path::Path};

pub(super) fn safe_build(build: &str) -> bool {
    !matches!(
        build.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) && !build.is_empty()
        && build.len() <= 100
        && build
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub(super) fn regular_path(path: &Path) -> Result<(), StartupError> {
    for ancestor in path.ancestors() {
        let meta =
            fs::symlink_metadata(ancestor).map_err(|e| format!("{}: {e}", ancestor.display()))?;
        if meta.file_attributes() & 0x400 != 0 {
            return Err(format!("reparse point refused: {}", ancestor.display()).into());
        }
    }
    if !path.is_file() {
        return Err(format!("regular file required: {}", path.display()).into());
    }
    Ok(())
}
