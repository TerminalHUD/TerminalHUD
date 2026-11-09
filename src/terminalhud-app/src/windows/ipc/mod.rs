//! Authenticated, bounded, local command transport.
pub(super) mod accept;
mod client;
mod connection;
mod operation;
pub(super) mod read;
mod server;
pub(super) mod write;
const IO_TIMEOUT: u32 = 1500;
const UI_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
pub use client::{call, call_for_process, call_request, connect, exchange};
pub(crate) use server::Server;

#[cfg(test)]
mod transport_tests;
