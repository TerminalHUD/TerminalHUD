use hudswitch_windows::HotkeyBinding;
use terminalhud_core::config::{Key, Modifier, Shortcut};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, VK_OEM_3,
};

pub(super) fn binding(id: i32, shortcut: &Shortcut) -> HotkeyBinding {
    let modifiers = shortcut.modifiers().iter().fold(0, |bits, modifier| {
        bits | match modifier {
            Modifier::Win => MOD_WIN,
            Modifier::Ctrl => MOD_CONTROL,
            Modifier::Alt => MOD_ALT,
            Modifier::Shift => MOD_SHIFT,
        }
    });
    let virtual_key = match shortcut.key() {
        Key::Backtick => u32::from(VK_OEM_3),
        Key::Letter(c) => c as u32,
        Key::Digit(n) => u32::from(b'0' + n),
        Key::Function(n) => 0x70 + u32::from(n - 1),
    };
    HotkeyBinding {
        id,
        modifiers,
        virtual_key,
    }
}
