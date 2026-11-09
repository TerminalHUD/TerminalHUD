use super::StartupError;
use std::{fs, os::windows::fs::MetadataExt, path::Path};

pub(super) fn safe_build(build: &str) -> bool {
    !build.is_empty()
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
