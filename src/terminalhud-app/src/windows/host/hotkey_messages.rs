//! A thread message hook preserves registered hotkeys inside native modal menus.
//! This observes this UI thread's queue only; it is not a keyboard/global hook.
use super::{UiSender, hotkeys};
use std::{cell::RefCell, io, ptr};
use windows_sys::Win32::{
    Foundation::*, System::Threading::GetCurrentThreadId, UI::WindowsAndMessaging::*,
};
thread_local! { static SENDER: RefCell<Option<UiSender>> = const { RefCell::new(None) }; }
pub(super) struct HotkeyMessages(HHOOK);
impl HotkeyMessages {
    pub(super) fn install(ui: UiSender) -> io::Result<Self> {
        let hook = unsafe {
            SetWindowsHookExW(
                WH_GETMESSAGE,
                Some(receive),
                ptr::null_mut(),
                GetCurrentThreadId(),
            )
        };
        if hook.is_null() {
            return Err(io::Error::last_os_error());
        }
        SENDER.with(|sender| *sender.borrow_mut() = Some(ui));
        Ok(Self(hook))
    }
}
unsafe extern "system" fn receive(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam == PM_REMOVE as usize {
        let message = unsafe { &mut *(lparam as *mut MSG) };
        if message.hwnd.is_null() && message.message == WM_HOTKEY {
            SENDER.with(|sender| {
                if let Some(ui) = sender.borrow().as_ref()
                    && hotkeys::enqueue(message.wParam, ui)
                {
                    message.message = WM_NULL;
                }
            });
        }
    }
    unsafe { CallNextHookEx(ptr::null_mut(), code, wparam, lparam) }
}
impl Drop for HotkeyMessages {
    fn drop(&mut self) {
        unsafe {
            UnhookWindowsHookEx(self.0);
        }
        SENDER.with(|sender| sender.borrow_mut().take());
    }
}
#[cfg(test)]
#[path = "hotkey_message_tests.rs"]
mod tests;
