use super::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

pub(super) struct HiddenWindow {
    pub(super) hwnd: HWND,
    class: Vec<u16>,
    instance: HINSTANCE,
}

impl HiddenWindow {
    pub(super) fn new() -> Self {
        static NEXT_CLASS: AtomicUsize = AtomicUsize::new(1);
        let class: Vec<_> = format!(
            "HudSwitch.AcquisitionRegression.{}.{}\0",
            std::process::id(),
            NEXT_CLASS.fetch_add(1, Ordering::Relaxed)
        )
        .encode_utf16()
        .collect();
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        let description = WNDCLASSW {
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            ..unsafe { std::mem::zeroed() }
        };
        assert_ne!(unsafe { RegisterClassW(&description) }, 0);
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                100,
                100,
                null_mut(),
                null_mut(),
                instance,
                std::ptr::null(),
            )
        };
        assert!(!hwnd.is_null());
        assert_eq!(unsafe { IsWindowVisible(hwnd) }, 0);
        Self {
            hwnd,
            class,
            instance,
        }
    }

    pub(super) fn receipt(&self) -> LaunchedWindow {
        let mut process = null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    GetCurrentProcess(),
                    GetCurrentProcess(),
                    &mut process,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        LaunchedWindow::claim(self.hwnd, unsafe { OwnedHandle::from_raw_handle(process) }).unwrap()
    }
}

impl Drop for HiddenWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
            UnregisterClassW(self.class.as_ptr(), self.instance);
        }
    }
}
