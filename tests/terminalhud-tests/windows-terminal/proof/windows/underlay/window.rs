use super::*;
use std::{
    ptr::{null, null_mut},
    sync::mpsc,
    thread,
};
use terminalhud_windows_terminal::native::fresh_guid;
use windows_sys::Win32::{Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW};

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

pub(super) fn start() -> Result<Underlay> {
    let name: Vec<_> = format!("TerminalHUD.TER9.Underlay.{}\0", fresh_guid()?)
        .encode_utf16()
        .collect();
    let class = name[..name.len() - 1].to_vec();
    let thread_class = class.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    let (stop, stopped) = mpsc::channel();
    let thread = thread::spawn(move || unsafe {
        let instance = GetModuleHandleW(null());
        let brush = CreateSolidBrush(40 | (160 << 8) | (220 << 16));
        let class = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            hbrBackground: brush,
            ..std::mem::zeroed()
        };
        if RegisterClassW(&class) == 0 {
            let _ = sender.send((0, 0));
            DeleteObject(brush);
            return;
        }
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST,
            name.as_ptr(),
            name.as_ptr(),
            WS_POPUP,
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
        }
        let thread_id = windows_sys::Win32::System::Threading::GetCurrentThreadId();
        let identity = Identity {
            process: windows_sys::Win32::System::Threading::GetCurrentProcessId(),
            thread: thread_id,
            class: thread_class,
        };
        let _ = sender.send((hwnd as isize, thread_id));
        if !hwnd.is_null() {
            run::pump(hwnd, &identity, stopped);
        }
        UnregisterClassW(name.as_ptr(), instance);
        DeleteObject(brush);
    });
    let (hwnd, owner_thread) = receiver.recv()?;
    if hwnd == 0 {
        let _ = thread.join();
        return Err("owned underlay creation failed".into());
    }
    Ok(Underlay {
        hwnd,
        identity: Identity {
            process: unsafe { windows_sys::Win32::System::Threading::GetCurrentProcessId() },
            thread: owner_thread,
            class,
        },
        stop,
        thread: Some(thread),
    })
}
