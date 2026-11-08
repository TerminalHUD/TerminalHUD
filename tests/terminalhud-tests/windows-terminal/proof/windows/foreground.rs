//! Read-only foreground event observation during launch, including transient focus.
use super::Result;
use std::cell::RefCell;
use std::ptr::null_mut;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::{Accessibility::*, WindowsAndMessaging::*};

thread_local! { static EVENTS: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) }; }
unsafe extern "system" fn observe(
    _: HWINEVENTHOOK,
    _: u32,
    hwnd: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    EVENTS.with(|events| events.borrow_mut().push(hwnd as isize));
}

pub(super) struct Monitor {
    id: u32,
    thread: Option<JoinHandle<Vec<isize>>>,
}
impl Monitor {
    pub(super) fn start() -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let thread = thread::spawn(move || unsafe {
            let mut message = std::mem::zeroed();
            PeekMessageW(&mut message, null_mut(), 0, 0, PM_NOREMOVE);
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                null_mut(),
                Some(observe),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );
            let id = GetCurrentThreadId();
            let _ = sender.send(if hook.is_null() { None } else { Some(id) });
            if !hook.is_null() {
                while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
                UnhookWinEvent(hook);
            }
            EVENTS.with(|events| events.take())
        });
        match receiver.recv()? {
            Some(id) => Ok(Self {
                id,
                thread: Some(thread),
            }),
            None => {
                let _ = thread.join();
                Err("SetWinEventHook refused foreground monitoring".into())
            }
        }
    }
    pub(super) fn finish(mut self) -> Result<Vec<isize>> {
        self.stop()
    }
    fn stop(&mut self) -> Result<Vec<isize>> {
        let Some(thread) = self.thread.take() else {
            return Ok(Vec::new());
        };
        if unsafe { PostThreadMessageW(self.id, WM_QUIT, 0, 0) } == 0 {
            return Err("foreground monitor stop failed".into());
        }
        thread
            .join()
            .map_err(|_| "foreground monitor panicked".into())
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
