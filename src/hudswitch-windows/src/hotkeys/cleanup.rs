use super::{HotkeyFailure, Hotkeys, ids};
use std::ptr::null_mut;
use windows_sys::Win32::{Foundation::GetLastError, UI::Input::KeyboardAndMouse::UnregisterHotKey};

impl Hotkeys {
    /// Unregister owned bindings; retain failed entries and their ID reservations.
    /// Success is idempotent. Call on the owning thread before its pump exits.
    pub fn unregister(&mut self) -> Result<(), Vec<HotkeyFailure>> {
        let mut failures = Vec::new();
        self.bindings.retain(|&binding| {
            if unsafe { UnregisterHotKey(null_mut(), binding.id) } == 0 {
                failures.push(HotkeyFailure::Win32 {
                    binding,
                    operation: "UnregisterHotKey",
                    code: unsafe { GetLastError() },
                });
                true
            } else {
                ids::release(binding.id);
                false
            }
        });
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

impl Drop for Hotkeys {
    fn drop(&mut self) {
        // On failure, keep the thread-local ID reserved rather than reuse it.
        let _ = self.unregister();
    }
}
