use super::{HotkeyBinding, Hotkeys};
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum HotkeyFailure {
    Invalid {
        binding: HotkeyBinding,
        reason: &'static str,
    },
    Win32 {
        binding: HotkeyBinding,
        operation: &'static str,
        code: u32,
    },
}

impl fmt::Display for HotkeyFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HotkeyFailure {}

/// A failed registration, its rollback failures, and any remaining owned keys.
/// Retry `remaining.as_mut().unwrap().unregister()` when cleanup is incomplete.
#[derive(Debug)]
pub struct HotkeyRegistrationError {
    pub failure: HotkeyFailure,
    pub cleanup_failures: Vec<HotkeyFailure>,
    pub remaining: Option<Hotkeys>,
}

impl From<HotkeyFailure> for HotkeyRegistrationError {
    fn from(failure: HotkeyFailure) -> Self {
        Self {
            failure,
            cleanup_failures: Vec::new(),
            remaining: None,
        }
    }
}
impl fmt::Display for HotkeyRegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}; rollback failures: {:?}",
            self.failure, self.cleanup_failures
        )
    }
}
impl std::error::Error for HotkeyRegistrationError {}
