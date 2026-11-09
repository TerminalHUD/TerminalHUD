//! Strict JSON candidates; no filesystem access or runtime application.
mod codec;
mod policy;
mod wire;

pub use codec::{decode, encode, replace_from_json};

#[cfg(test)]
mod tests;
