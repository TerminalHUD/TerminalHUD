use super::{NativeError, error::last};
use crate::WindowCandidate;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub fn windows() -> Result<Vec<WindowCandidate>, NativeError> {
    unsafe extern "system" fn collect(hwnd: HWND, context: LPARAM) -> i32 {
        let windows = unsafe { &mut *(context as *mut Vec<WindowCandidate>) };
        let mut class = [0u16; 128];
        let length = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
        if length <= 0 {
            return 1;
        }
        let class = String::from_utf16_lossy(&class[..length as usize]);
        if class != "CASCADIA_HOSTING_WINDOW_CLASS" {
            return 1;
        }
        let length = unsafe { GetWindowTextLengthW(hwnd) };
        let mut title = vec![0u16; length.max(0) as usize + 1];
        let length = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32) };
        let mut pid = 0;
        if unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } == 0 {
            return 1;
        }
        windows.push(WindowCandidate {
            handle: hwnd as isize,
            process_id: pid,
            class,
            title: String::from_utf16_lossy(&title[..length.max(0) as usize]),
            existed_before_launch: false,
        });
        1
    }
    let mut windows = Vec::new();
    if unsafe { EnumWindows(Some(collect), &mut windows as *mut _ as LPARAM) } == 0 {
        return Err(last("EnumWindows Terminal candidates"));
    }
    Ok(windows)
}
