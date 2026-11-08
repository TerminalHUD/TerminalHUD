//! Native launch/correlation primitives; no profile installation or style changes.
mod error;
mod identity;
mod launch;
mod process;
mod verify;
mod windows;
pub use error::NativeError;
pub use identity::fresh_guid;
pub use launch::{LauncherProcess, start};
pub use process::{ProcessIdentity, ancestry};
pub use verify::{VerifiedWindow, verify};
pub use windows::windows;
