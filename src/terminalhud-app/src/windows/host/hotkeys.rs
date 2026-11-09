//! Intended defaults only. A conflict leaves tray/IPC usable without rebinding.
use super::{UiMessage, UiSender};
use hudswitch_windows::{HotkeyBinding, HotkeyFailure, Hotkeys};
use terminalhud_core::Command;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOD_SHIFT, MOD_WIN, VK_OEM_3};
const INTERACTION: i32 = 0x5448;
const VISIBILITY: i32 = 0x5449;
pub(super) fn register() -> (Option<Hotkeys>, u32) {
    match Hotkeys::register(&[
        HotkeyBinding {
            id: INTERACTION,
            modifiers: MOD_WIN,
            virtual_key: u32::from(VK_OEM_3),
        },
        HotkeyBinding {
            id: VISIBILITY,
            modifiers: MOD_WIN | MOD_SHIFT,
            virtual_key: u32::from(VK_OEM_3),
        },
    ]) {
        Ok(keys) => (Some(keys), 0),
        Err(mut error) => {
            eprintln!(
                "TerminalHUD Win+Backtick / Win+Shift+Backtick registration failed: {error:?}; tray and IPC remain available; bindings unchanged"
            );
            let code = match error.failure {
                HotkeyFailure::Win32 { code, .. } => code,
                _ => u32::MAX,
            };
            // Retain any batch rollback failures until normal UI-thread teardown.
            (error.remaining.take(), code)
        }
    }
}
pub(super) fn enqueue(id: usize, ui: &UiSender) -> bool {
    let command = match id as i32 {
        INTERACTION => Command::InteractionToggle,
        VISIBILITY => Command::VisibilityToggle,
        _ => return false,
    };
    if let Err(error) = ui.send(UiMessage::Request {
        command,
        reply: None,
    }) {
        eprintln!("TerminalHUD hotkey command not queued: {error}");
    }
    true
}
