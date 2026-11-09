//! Win32 implementation of the portable HudSwitch backend contract.
//!
//! The caller owns the event/controller thread and supplies an intended-target
//! launcher. This backend has no permanent event loop and never searches for an
//! arbitrary existing user window. See `docs/windows-backend.md`.

#![cfg(windows)]

mod backend;
mod error;
mod hotkeys;

pub use backend::{
    Focus, LaunchedWindow, Launcher, Original, RecoveryClaim, RecoveryWatch, Target, WindowsBackend,
};
pub use error::Error;
pub use hotkeys::{HotkeyBinding, HotkeyFailure, HotkeyRegistrationError, Hotkeys};
