use super::Result;
use std::{
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub(super) fn wait(mut condition: impl FnMut() -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !condition() {
        if Instant::now() >= deadline {
            return Err("native observation timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
pub(super) fn fullscreen(hwnd: HWND) -> Result<()> {
    let mut rectangle = unsafe { std::mem::zeroed::<RECT>() };
    if unsafe { GetWindowRect(hwnd, &mut rectangle) } == 0 {
        return Err("cannot observe Terminal rectangle".into());
    }
    let expected = (0, 0, unsafe { GetSystemMetrics(SM_CXSCREEN) }, unsafe {
        GetSystemMetrics(SM_CYSCREEN)
    });
    let actual = (
        rectangle.left,
        rectangle.top,
        rectangle.right,
        rectangle.bottom,
    );
    println!("Terminal fullscreen rectangle={actual:?}, primary={expected:?}");
    if actual != expected {
        return Err("Terminal is not primary-monitor fullscreen".into());
    }
    Ok(())
}
pub(super) fn passive(hwnd: HWND) -> Result<()> {
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;
    let required = WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT;
    if style & required != required
        || unsafe { IsWindowVisible(hwnd) } == 0
        || unsafe { GetForegroundWindow() } == hwnd
    {
        return Err("actual Passive style/visibility/focus invariant failed".into());
    }
    Ok(())
}
