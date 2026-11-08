use super::super::*;

#[test]
fn invalid_bindings_and_duplicate_ids_or_normalized_chords_are_rejected() {
    let first = binding(1, VK_F23);
    for invalid in [
        HotkeyBinding { id: -1, ..first },
        HotkeyBinding {
            id: 0xc000,
            ..first
        },
        HotkeyBinding {
            modifiers: 0x8000,
            ..first
        },
        HotkeyBinding {
            virtual_key: 0,
            ..first
        },
        HotkeyBinding {
            virtual_key: 0xff,
            ..first
        },
        binding(1, VK_F12),
    ] {
        assert!(matches!(
            Hotkeys::register(&[invalid]).unwrap_err().failure,
            HotkeyFailure::Invalid { .. }
        ));
    }
    for second in [
        binding(1, VK_F24),
        HotkeyBinding {
            id: 2,
            modifiers: first.modifiers | MOD_NOREPEAT,
            ..first
        },
    ] {
        let error = Hotkeys::register(&[first, second]).unwrap_err();
        assert!(
            matches!(error.failure, HotkeyFailure::Invalid { binding, .. } if binding == second)
        );
        assert!(error.cleanup_failures.is_empty());
        assert!(error.remaining.is_none());
    }
}
