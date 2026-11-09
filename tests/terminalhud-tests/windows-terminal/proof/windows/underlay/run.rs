use super::{Identity, is_owned};
use std::{sync::mpsc, time::Duration};
use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

pub(super) fn pump(hwnd: HWND, identity: &Identity, stop: mpsc::Receiver<()>) {
    loop {
        let mut message = unsafe { std::mem::zeroed() };
        while unsafe { PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
            if message.message == WM_QUIT {
                println!("PHASE sink-closed: owning thread received quit");
                return;
            }
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        match stop.recv_timeout(Duration::from_millis(10)) {
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            _ => {
                if is_owned(hwnd, identity) {
                    if unsafe { DestroyWindow(hwnd) } == 0 {
                        eprintln!("PHASE sink-cleanup failed: DestroyWindow refused");
                    } else {
                        println!("PHASE sink-cleanup: owned window destroyed");
                    }
                } else {
                    println!("PHASE sink-cleanup: identity no longer matches; no window touched");
                }
                return;
            }
        }
    }
}
