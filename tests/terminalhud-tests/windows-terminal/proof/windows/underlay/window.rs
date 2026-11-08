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
        WM_LBUTTONDOWN => {
            unsafe {
                SetWindowLongPtrW(
                    hwnd,
                    GWLP_USERDATA,
                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) + 1,
                );
            }
            0
        }
        value if value == WM_APP + 1 => unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) },
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

pub(super) fn start() -> Result<Underlay> {
    let name: Vec<_> = format!("TerminalHUD.TER9.Underlay.{}\0", fresh_guid()?)
        .encode_utf16()
        .collect();
    let (sender, receiver) = mpsc::sync_channel(1);
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
            let _ = sender.send(0);
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
        let _ = sender.send(hwnd as isize);
        if !hwnd.is_null() {
            let mut message = std::mem::zeroed();
            while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        UnregisterClassW(name.as_ptr(), instance);
        DeleteObject(brush);
    });
    let hwnd = receiver.recv()?;
    if hwnd == 0 {
        let _ = thread.join();
        return Err("owned underlay creation failed".into());
    }
    Ok(Underlay {
        hwnd,
        thread: Some(thread),
    })
}
