//! Platform application composition; portable cores contain no Windows types.
pub mod development;
pub mod host;
pub mod ipc;
mod native;
pub mod security;
mod tray;
pub use host::{HostOptions, MENU_PROBE_MESSAGE, UiSender, run_host, run_host_args};
pub use tray::tray;
