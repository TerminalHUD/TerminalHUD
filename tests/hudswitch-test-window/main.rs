//! Purpose-built native target; no terminal or application behavior.
#[cfg(not(windows))]
fn main() {
    eprintln!("The generic test window requires Windows.");
}

#[cfg(windows)]
fn main() {
    use std::io::Write;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

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
                let count = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, count + 1);
                }
                0
            }
            value if value == WM_APP + 1 => unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) },
            value if value == WM_APP + 2 => {
                // Test-only unrelated style mutation, preserving backend-owned bits.
                let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
                unsafe {
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_TOOLWINDOW as isize);
                }
                1
            }
            _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
        }
    }

    let arguments: Vec<_> = std::env::args().collect();
    let passive = arguments.iter().any(|argument| argument == "--passive");
    let layered = arguments.iter().any(|argument| argument == "--layered");
    let name: Vec<_> = "HudSwitch.GenericFixture\0".encode_utf16().collect();
    let title: Vec<_> = format!("HudSwitch test window {}\0", std::process::id())
        .encode_utf16()
        .collect();
    unsafe {
        assert_ne!(SetProcessDPIAware(), 0);
        let instance = GetModuleHandleW(null());
        let class = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            ..std::mem::zeroed()
        };
        assert_ne!(RegisterClassW(&class), 0);
        let hwnd = CreateWindowExW(
            if layered { WS_EX_LAYERED } else { 0 },
            name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            100,
            100,
            480,
            320,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        assert!(!hwnd.is_null());
        if layered {
            assert_ne!(
                SetLayeredWindowAttributes(hwnd, 0x00010203, 211, LWA_ALPHA | LWA_COLORKEY),
                0
            );
        }
        ShowWindow(hwnd, if passive { SW_SHOWNOACTIVATE } else { SW_SHOW });
        UpdateWindow(hwnd);
        println!("READY {} {}", std::process::id(), hwnd as usize);
        std::io::stdout().flush().unwrap();
        let mut message = std::mem::zeroed();
        loop {
            let result = GetMessageW(&mut message, null_mut(), 0, 0);
            if result <= 0 {
                break;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
