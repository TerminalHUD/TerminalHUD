use super::{HotkeyBinding, HotkeyFailure, HotkeyRegistrationError, Hotkeys, binding, ids};
use std::{marker::PhantomData, ptr::null_mut};
use windows_sys::Win32::{
    Foundation::GetLastError,
    UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, RegisterHotKey},
};

impl Hotkeys {
    /// Register a batch on this thread, always suppressing keyboard auto-repeat.
    /// Validate the whole batch before mutation; unwind only owned successes.
    pub fn register(bindings: &[HotkeyBinding]) -> Result<Self, HotkeyRegistrationError> {
        binding::validate(bindings)?;
        let mut owned = Self {
            bindings: Vec::with_capacity(bindings.len()),
            _thread: PhantomData,
        };
        ids::reserve(bindings)?;
        for (index, &binding) in bindings.iter().enumerate() {
            let registered = unsafe {
                RegisterHotKey(
                    null_mut(),
                    binding.id,
                    binding.modifiers | MOD_NOREPEAT,
                    binding.virtual_key,
                )
            };
            if registered == 0 {
                let failure = HotkeyFailure::Win32 {
                    binding,
                    operation: "RegisterHotKey",
                    code: unsafe { GetLastError() },
                };
                for pending in &bindings[index..] {
                    ids::release(pending.id);
                }
                let cleanup_failures = owned.unregister().err().unwrap_or_default();
                return Err(HotkeyRegistrationError {
                    failure,
                    cleanup_failures,
                    remaining: (!owned.bindings.is_empty()).then_some(owned),
                });
            }
            owned.bindings.push(binding);
        }
        Ok(owned)
    }
}
