//! Native launch/correlation primitives; no profile installation or style changes.
mod ancestry;
mod command;
mod error;
mod identity;
mod launch;
mod process;
mod support;
mod verify;
mod windows;
pub use ancestry::ancestry;
pub use error::NativeError;
pub use identity::fresh_guid;
pub use launch::{LauncherProcess, start, start_with_command};
pub use process::ProcessIdentity;
pub use verify::{VerifiedWindow, verify};
pub use windows::windows;
