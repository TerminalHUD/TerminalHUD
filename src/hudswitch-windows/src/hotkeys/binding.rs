use super::HotkeyFailure;
use std::collections::BTreeSet;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, VK_F12,
};

/// Application ID, Win32 modifier bits, and virtual-key code; no product policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub id: i32,
    pub modifiers: u32,
    pub virtual_key: u32,
}

pub(super) fn validate(bindings: &[HotkeyBinding]) -> Result<(), HotkeyFailure> {
    let mut ids = BTreeSet::new();
    let mut chords = BTreeSet::new();
    for &binding in bindings {
        let reason = if !(0..=0xbfff).contains(&binding.id) {
            Some("application hotkey ID must be in 0..=0xBFFF")
        } else if binding.modifiers & !(MOD_ALT | MOD_CONTROL | MOD_SHIFT | MOD_WIN | MOD_NOREPEAT)
            != 0
        {
            Some("unknown modifier bits")
        } else if !(1..=0xfe).contains(&binding.virtual_key) {
            Some("virtual-key code must be in 1..=0xFE")
        } else if binding.virtual_key == u32::from(VK_F12) {
            Some("F12 is reserved for the debugger")
        } else if !ids.insert(binding.id) {
            Some("duplicate hotkey ID")
        } else if !chords.insert((binding.modifiers & !MOD_NOREPEAT, binding.virtual_key)) {
            Some("duplicate hotkey chord")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(HotkeyFailure::Invalid { binding, reason });
        }
    }
    Ok(())
}
