use std::{
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
fn belongs(hwnd: HWND, pid: u32) -> bool {
    let mut owner = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut owner);
    }
    owner == pid
}
pub(super) fn host(pid: u32) -> HWND {
    let class = wide("TerminalHUD.Prototype.Host");
    let mut after = std::ptr::null_mut();
    loop {
        let found = unsafe { FindWindowExW(HWND_MESSAGE, after, class.as_ptr(), std::ptr::null()) };
        assert!(!found.is_null(), "owned message-only host missing");
        if belongs(found, pid) {
            return found;
        }
        after = found;
    }
}
pub(super) fn visible(pid: u32) -> Vec<HWND> {
    struct Search {
        pid: u32,
        found: Vec<HWND>,
    }
    unsafe extern "system" fn visit(hwnd: HWND, param: LPARAM) -> windows_sys::core::BOOL {
        let search = unsafe { &mut *(param as *mut Search) };
        if belongs(hwnd, search.pid) && unsafe { IsWindowVisible(hwnd) } != 0 {
            search.found.push(hwnd);
        }
        1
    }
    let mut search = Search {
        pid,
        found: Vec::new(),
    };
    unsafe {
        EnumWindows(Some(visit), (&mut search as *mut Search) as isize);
    }
    search.found
}
pub(super) fn wait_menu(pid: u32) -> HWND {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(hwnd) = visible(pid).into_iter().find(|h| {
            let mut class = [0u16; 64];
            let n = unsafe { GetClassNameW(*h, class.as_mut_ptr(), 64) };
            String::from_utf16_lossy(&class[..n as usize]) == "#32768"
        }) {
            return hwnd;
        }
        assert!(
            Instant::now() < deadline,
            "owned native popup did not appear"
        );
        thread::sleep(Duration::from_millis(20));
    }
}
pub(super) fn open_menu(pid: u32) -> HWND {
    let hwnd = host(pid);
    assert!(belongs(hwnd, pid));
    assert_ne!(
        unsafe { PostMessageW(hwnd, terminalhud_app::windows::MENU_PROBE_MESSAGE, 0, 0) },
        0
    );
    wait_menu(pid)
}
