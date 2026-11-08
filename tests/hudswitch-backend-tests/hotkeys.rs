#![cfg(windows)]

use hudswitch_windows::{HotkeyBinding, HotkeyFailure, Hotkeys};
use std::sync::Mutex;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, VK_F12, VK_F23, VK_F24,
};

mod hotkeys {
    pub(super) mod blocker;
    mod cleanup;
    mod lifecycle;
    mod validation;
}

static SERIAL: Mutex<()> = Mutex::new(());

fn binding(id: i32, key: u16) -> HotkeyBinding {
    HotkeyBinding {
        id,
        modifiers: MOD_CONTROL | MOD_ALT | MOD_SHIFT,
        virtual_key: u32::from(key),
    }
}
