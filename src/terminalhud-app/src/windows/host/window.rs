use super::{
    super::native::{last, wide},
    context::HostContext,
    messages::window_proc,
};
use std::{io, mem, ptr};
use windows_sys::Win32::{
    Foundation::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*,
};
pub(super) struct HostWindow(pub(super) HWND);
impl HostWindow {
    pub(super) fn create(context: &HostContext) -> io::Result<Self> {
        let class = wide("TerminalHUD.Prototype.Host");
        let module = unsafe { GetModuleHandleW(ptr::null()) };
        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: module,
            lpszClassName: class.as_ptr(),
            ..unsafe { mem::zeroed() }
        };
        if unsafe { RegisterClassW(&wc) } == 0
            && unsafe { GetLastError() } != ERROR_CLASS_ALREADY_EXISTS
        {
            return Err(last("RegisterClass"));
        }
        let name = wide("TerminalHUD prototype controller (message-only)");
        // The boxed context outlives DestroyWindow; WM_NCCREATE only stores its address.
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                ptr::null_mut(),
                module,
                (context as *const HostContext).cast(),
            )
        };
        if hwnd.is_null() {
            return Err(last("CreateWindowEx message host"));
        }
        Ok(Self(hwnd))
    }
}
impl Drop for HostWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.0);
        }
    }
}
