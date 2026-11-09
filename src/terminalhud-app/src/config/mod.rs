//! Validated JSON, bounded settings files and platform document operations.
#[cfg(any(windows, test))]
pub(crate) mod bootstrap;
mod codec;
#[cfg(any(windows, test))]
pub(crate) mod hotkey_transaction;
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
#[cfg(any(windows, test))]
pub(crate) use file_create::load_or_create;
pub(crate) use file_read::validate_file;
pub(crate) use platform::{open_document, settings_path};
