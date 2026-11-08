//! Authenticated, bounded, prototype-only local command transport.
mod accept;
mod client;
mod connection;
mod operation;
mod read;
mod server;
mod write;
const IO_TIMEOUT: u32 = 1500;
const UI_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
pub use client::{call, connect, exchange};
pub(crate) use server::Server;
