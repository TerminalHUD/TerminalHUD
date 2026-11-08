use super::super::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};

#[test]
fn cleanup_failure_remains_owned_and_reserved_until_successful_retry() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let chord = binding(1, VK_F23);
    let mut owned = Hotkeys::register(&[chord]).expect("test chord unavailable");
    // Deliberately invalidate only this test's own native registration.
    assert_ne!(
        unsafe { UnregisterHotKey(std::ptr::null_mut(), chord.id) },
        0
    );
    let errors = owned.unregister().unwrap_err();
    assert!(
        matches!(&errors[..], [HotkeyFailure::Win32 { binding, operation: "UnregisterHotKey", code }]
        if *binding == chord && *code == windows_sys::Win32::Foundation::ERROR_HOTKEY_NOT_REGISTERED)
    );
    assert_eq!(owned.bindings(), &[chord]);
    assert!(
        matches!(Hotkeys::register(&[binding(1, VK_F24)]).unwrap_err().failure,
        HotkeyFailure::Invalid { reason, .. } if reason == "hotkey ID already owned on this thread")
    );
    // Restore that same test-owned registration so explicit cleanup can retry.
    assert_ne!(
        unsafe {
            RegisterHotKey(
                std::ptr::null_mut(),
                chord.id,
                chord.modifiers | MOD_NOREPEAT,
                chord.virtual_key,
            )
        },
        0
    );
    owned.unregister().unwrap();
    assert!(owned.bindings().is_empty());
    drop(Hotkeys::register(&[binding(1, VK_F24)]).unwrap());
}

#[test]
fn failed_drop_does_not_release_its_thread_id_for_reuse() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    std::thread::spawn(|| {
        let chord = binding(1, VK_F23);
        let owned = Hotkeys::register(&[chord]).expect("test chord unavailable");
        // Invalidate only this test-owned key to exercise the Drop failure path.
        assert_ne!(
            unsafe { UnregisterHotKey(std::ptr::null_mut(), chord.id) },
            0
        );
        drop(owned);
        assert!(matches!(
            Hotkeys::register(&[binding(1, VK_F24)]).unwrap_err().failure,
            HotkeyFailure::Invalid { reason, .. }
                if reason == "hotkey ID already owned on this thread"
        ));
        // The dedicated test thread exits with no native registration remaining.
    })
    .join()
    .unwrap();
}
