use super::super::hotkey_binding::binding;
use super::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, VK_OEM_3,
};
#[test]
fn symbolic_shortcuts_translate_to_native_keys() {
    for (shortcut, modifiers, key) in [
        ("Win+Backtick", MOD_WIN, u32::from(VK_OEM_3)),
        ("Ctrl+Alt+A", MOD_CONTROL | MOD_ALT, 0x41),
        ("Shift+9", MOD_SHIFT, 0x39),
        ("Win+F1", MOD_WIN, 0x70),
        ("Win+F24", MOD_WIN, 0x87),
    ] {
        let value = binding(1, &Shortcut::parse(shortcut).unwrap());
        assert_eq!((value.modifiers, value.virtual_key), (modifiers, key));
    }
}
