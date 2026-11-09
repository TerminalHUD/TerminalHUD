//! Native thread-queue messages only; no key registration/input or visible UI.
use super::*;
use crate::windows::host::UiMessage;
use std::{mem, sync::mpsc};
#[test]
fn thread_hotkeys_enqueue_once_only_when_removed_and_stop_after_unhook() {
    let mut message: MSG = unsafe { mem::zeroed() };
    unsafe {
        PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_NOREMOVE);
    }
    let (sender, receiver) = mpsc::sync_channel(4);
    let hook = HotkeyMessages::install(UiSender::new(sender, 0)).unwrap();
    let thread = unsafe { GetCurrentThreadId() };
    assert_ne!(
        unsafe { PostThreadMessageW(thread, WM_HOTKEY, 0x5448, 0) },
        0
    );
    assert_ne!(
        unsafe {
            PeekMessageW(
                &mut message,
                ptr::null_mut(),
                WM_HOTKEY,
                WM_HOTKEY,
                PM_NOREMOVE,
            )
        },
        0
    );
    assert!(receiver.try_recv().is_err());
    assert_ne!(
        unsafe {
            PeekMessageW(
                &mut message,
                ptr::null_mut(),
                WM_HOTKEY,
                WM_HOTKEY,
                PM_REMOVE,
            )
        },
        0
    );
    assert_eq!(message.message, WM_NULL);
    assert!(matches!(
        receiver.try_recv().unwrap(),
        UiMessage::Request {
            command: terminalhud_core::Command::InteractionToggle,
            reply: None
        }
    ));
    assert!(receiver.try_recv().is_err());
    // Another component's ID is passed through, even on our thread.
    assert_ne!(
        unsafe { PostThreadMessageW(thread, WM_HOTKEY, 0x1234, 0) },
        0
    );
    unsafe {
        PeekMessageW(
            &mut message,
            ptr::null_mut(),
            WM_HOTKEY,
            WM_HOTKEY,
            PM_REMOVE,
        );
    }
    assert_eq!(message.message, WM_HOTKEY);
    assert!(receiver.try_recv().is_err());
    drop(hook);
    assert_ne!(
        unsafe { PostThreadMessageW(thread, WM_HOTKEY, 0x5448, 0) },
        0
    );
    unsafe {
        PeekMessageW(
            &mut message,
            ptr::null_mut(),
            WM_HOTKEY,
            WM_HOTKEY,
            PM_REMOVE,
        );
    }
    assert_eq!(message.message, WM_HOTKEY);
    while unsafe { PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {}
}
