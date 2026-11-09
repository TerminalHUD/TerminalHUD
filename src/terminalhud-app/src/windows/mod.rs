//! Platform application composition; portable cores contain no Windows types.
pub mod development;
pub mod host;
pub mod ipc;
mod native;
pub mod process_output;
pub(crate) mod process_spawn;
pub mod security;
pub mod startup;
mod tray;
pub use host::{HostOptions, MENU_PROBE_MESSAGE, UiSender, run_host, run_host_args};
pub use tray::tray;
