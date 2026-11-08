use super::super::*;
use super::blocker::Blocker;

#[test]
fn explicit_cleanup_and_drop_release_chords_and_ids() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let chord = binding(1, VK_F23);
    let mut owned = Hotkeys::register(&[chord]).expect("test chord unavailable");
    assert_eq!(owned.bindings(), &[chord]);
    owned.unregister().unwrap();
    owned.unregister().unwrap();
    assert!(owned.bindings().is_empty());
    // Another thread can claim the same chord only after native cleanup.
    drop(Blocker::start(chord));
    let owned = Hotkeys::register(&[chord]).unwrap();
    drop(owned);
    drop(Blocker::start(chord));
    // The per-thread ID reservation was released too.
    drop(Hotkeys::register(&[binding(1, VK_F24)]).unwrap());
}

#[test]
fn a_second_guard_cannot_reuse_an_owned_thread_id() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let first = binding(1, VK_F23);
    let owned = Hotkeys::register(&[first]).expect("test chord unavailable");
    let error = Hotkeys::register(&[binding(1, VK_F24)]).unwrap_err();
    assert!(
        matches!(error.failure, HotkeyFailure::Invalid { reason, .. }
        if reason == "hotkey ID already owned on this thread")
    );
    assert_eq!(owned.bindings(), &[first]);
    drop(owned);
    drop(Hotkeys::register(&[binding(1, VK_F24)]).unwrap());
}

#[test]
fn partial_registration_failure_unwinds_only_its_own_successes() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    let blocker = Blocker::start(binding(1, VK_F23));
    let batch = [binding(1, VK_F24), binding(2, VK_F23)];
    let error = Hotkeys::register(&batch).unwrap_err();
    assert!(
        matches!(error.failure, HotkeyFailure::Win32 { binding, operation: "RegisterHotKey", code }
        if binding == batch[1] && code == windows_sys::Win32::Foundation::ERROR_HOTKEY_ALREADY_REGISTERED)
    );
    assert!(error.cleanup_failures.is_empty());
    assert!(error.remaining.is_none());
    // Rollback freed the successful prefix and both reserved IDs.
    let mut prefix = Hotkeys::register(&[binding(1, VK_F24)]).unwrap();
    prefix.unregister().unwrap();
    // It did not unregister the other thread's deliberately conflicting key.
    assert!(
        matches!(Hotkeys::register(&[binding(2, VK_F23)]).unwrap_err().failure,
        HotkeyFailure::Win32 { code, .. }
        if code == windows_sys::Win32::Foundation::ERROR_HOTKEY_ALREADY_REGISTERED)
    );
    drop(blocker);
    drop(Hotkeys::register(&[binding(2, VK_F23)]).unwrap());
}
