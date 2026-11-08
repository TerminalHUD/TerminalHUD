//! Application transports and host-thread composition.

pub mod cli;
pub mod protocol;
pub mod prototype;

#[cfg(windows)]
pub mod windows;
