use super::{
    MENU_PROBE_MESSAGE, TIMER_DURATION, TIMER_QUIT, TIMER_REFRESH, WM_WAKE, context::HostContext,
};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};
pub(super) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        }
    }
    let context = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const HostContext;
    if !context.is_null() && message == MENU_PROBE_MESSAGE {
        unsafe { &*context }.show_probe_menu();
        return 0;
    }
    if !context.is_null()
        && (message == WM_WAKE
            || message == WM_CLOSE
            || (message == WM_TIMER
                && (wparam == TIMER_DURATION || wparam == TIMER_QUIT || wparam == TIMER_REFRESH)))
    {
        // Shared access only: actual controller access is guarded inside dispatch.
        let context = unsafe { &*context };
        if message == WM_TIMER && wparam == TIMER_REFRESH {
            context.refresh_requested.set(true);
        } else if message != WM_WAKE {
            context.close_requested.set(true);
        }
        context.dispatch();
        return 0;
    }
    if message == WM_NCDESTROY {
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        }
    }
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}
