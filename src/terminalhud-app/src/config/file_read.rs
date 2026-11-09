use super::{
    decode,
    file_type::{file_error, require_regular},
};
use std::{fs::File, io::Read, path::Path};
use terminalhud_core::config::Configuration;

pub(super) const MAX_SETTINGS_BYTES: u64 = 1024 * 1024;

pub(crate) fn validate_file(path: &Path) -> Result<Configuration, String> {
    read_file(path)
        .and_then(|json| decode(&json).map_err(|error| file_error(path, "validate", error)))
}

fn read_file(path: &Path) -> Result<String, String> {
    require_regular(path).map_err(|error| file_error(path, "read", error))?;
    let file = File::open(path).map_err(|error| file_error(path, "read", error))?;
    let metadata = file
        .metadata()
        .map_err(|error| file_error(path, "read", error))?;
    if !metadata.is_file() {
        return Err(file_error(path, "read", "expected a regular file"));
    }
    if metadata.len() > MAX_SETTINGS_BYTES {
        return Err(file_error(path, "read", "settings exceed the 1 MiB limit"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| file_error(path, "read", error))?;
    if bytes.len() as u64 > MAX_SETTINGS_BYTES {
        return Err(file_error(path, "read", "settings exceed the 1 MiB limit"));
    }
    String::from_utf8(bytes).map_err(|error| file_error(path, "read UTF-8", error))
}
