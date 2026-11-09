use std::{fs, io, path::Path};

pub(super) fn require_regular(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_file() {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "expected a regular settings file, not a directory, symlink or special file",
    ))
}

pub(super) fn file_error(path: &Path, operation: &str, error: impl std::fmt::Display) -> String {
    format!(
        "cannot {operation} configuration {}: {error}",
        path.display()
    )
}
