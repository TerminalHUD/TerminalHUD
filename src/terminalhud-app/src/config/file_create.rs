use super::{
    encode,
    file_type::{file_error, require_regular},
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};
use terminalhud_core::config::Configuration;

/// Explicit open may create defaults, but never truncates an existing file.
pub(crate) fn ensure_file(path: &Path) -> Result<(), String> {
    match require_regular(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(file_error(path, "open", error)),
    }
    let json =
        encode(&Configuration::default()).map_err(|error| file_error(path, "encode", error))?;
    let parent = path
        .parent()
        .ok_or_else(|| file_error(path, "create", "missing parent directory"))?;
    fs::create_dir_all(parent).map_err(|error| file_error(path, "create directory for", error))?;
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => file
            .write_all(json.as_bytes())
            .map_err(|error| file_error(path, "write new", error)),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            require_regular(path).map_err(|error| file_error(path, "open", error))
        }
        Err(error) => Err(file_error(path, "create", error)),
    }
}

/// Startup creates only a missing file, then validates the resulting contents.
#[cfg(any(windows, test))]
pub(crate) fn load_or_create(path: &Path) -> Result<Configuration, String> {
    ensure_file(path)?;
    super::validate_file(path)
}
