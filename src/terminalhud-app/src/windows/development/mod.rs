//! Explicit portable development composition; never discovers ordinary targets.
mod bridge;
mod config;
mod handshake;
mod launcher;
mod namespaces;
mod nonce;
mod runtime_profiles;
mod temporary;
pub use bridge::run_bridge;
pub(super) use config::Configuration;
pub(super) use launcher::DevelopmentLauncher;
use std::io;
pub(crate) use temporary::directory as temporary_directory;
fn error(value: impl std::fmt::Display) -> io::Error {
    io::Error::other(value.to_string())
}
