use std::{thread, time::Duration};
use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

/// Exercise the native popup's input path, never the application's callback.
/// UIA exposes no descendants for this popup on the validation desktop.
pub(super) fn interaction(pid: u32, menu: usize) {
    let menu = menu as HWND;
    for key in [0x28usize, 0x0d] {
        // Down, Return; messages target only this popup.
        post(menu, pid, WM_KEYDOWN, key, 1);
        // Return may dismiss the popup before its key-up can be delivered.
        if owned_popup(menu, pid) {
            post(menu, pid, WM_KEYUP, key, 0xc0000001u32 as isize);
        } else {
            assert_eq!(key, 0x0d, "popup closed before selection");
        }
        thread::sleep(Duration::from_millis(30));
    }
}
fn owned_popup(menu: HWND, pid: u32) -> bool {
    let mut owner = 0;
    let mut class = [0u16; 64];
    // SAFETY: output buffers are live; the system validates the foreign HWND.
    unsafe {
        if IsWindow(menu) == 0 || IsWindowVisible(menu) == 0 {
            return false;
        }
        GetWindowThreadProcessId(menu, &mut owner);
        let len = GetClassNameW(menu, class.as_mut_ptr(), class.len() as i32);
        owner == pid && String::from_utf16_lossy(&class[..len as usize]) == "#32768"
    }
}
fn post(menu: HWND, pid: u32, message: u32, key: usize, flags: isize) {
    assert!(
        owned_popup(menu, pid),
        "owned popup closed or identity changed"
    );
    assert_ne!(unsafe { PostMessageW(menu, message, key, flags) }, 0);
}
