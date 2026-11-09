//! Strict JSON candidates; no filesystem access or runtime application.
mod codec;
mod object;
mod policy;
mod wire;

pub use codec::{decode, encode, replace_from_json};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod shape_tests;

mod file_create;
mod file_read;
mod file_type;
mod platform;
pub(crate) use file_create::ensure_file;
pub(crate) use file_read::validate_file;
pub(crate) use platform::{open_document, settings_path};
