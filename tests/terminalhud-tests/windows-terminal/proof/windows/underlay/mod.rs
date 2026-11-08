//! A colored, owned primary-monitor focus/click sink; never a normal user window.
mod window;
use super::Result;
use std::thread::JoinHandle;
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub(super) struct Underlay {
    hwnd: isize,
    thread: Option<JoinHandle<()>>,
}
impl Underlay {
    pub(super) fn start() -> Result<Self> {
        window::start()
    }
    pub(super) fn hwnd(&self) -> HWND {
        self.hwnd as HWND
    }
}
impl Drop for Underlay {
    fn drop(&mut self) {
        unsafe {
            PostMessageW(self.hwnd(), WM_CLOSE, 0, 0);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
