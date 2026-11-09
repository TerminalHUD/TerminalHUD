use super::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, VK_OEM_3,
};
#[test]
fn unchanged_and_swapped_chords_need_no_registration() {
    let current = Configuration::default();
    let existing = [current.toggle_interaction(), current.toggle_visibility()];
    assert!(new_shortcuts(&existing, &current).is_empty());
    let swapped = Configuration::validate(terminalhud_core::config::Candidate {
        toggle_interaction: current.toggle_visibility().to_string(),
        toggle_visibility: current.toggle_interaction().to_string(),
        ..Default::default()
    })
    .unwrap();
    assert!(new_shortcuts(&existing, &swapped).is_empty());
    let changed = Configuration::validate(terminalhud_core::config::Candidate {
        toggle_interaction: "Ctrl+A".into(),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        new_shortcuts(&existing, &changed),
        vec![Shortcut::parse("Ctrl+A").unwrap()]
    );
}
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
