//! Read-only before/after observations of already-existing Terminal windows.
use super::Result;
use terminalhud_windows_terminal::native;
use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

pub(super) struct Witness(Vec<(isize, u32, isize, bool)>);
impl Witness {
    pub(super) fn capture() -> Result<Self> {
        let windows = native::windows()?;
        Ok(Self(
            windows
                .iter()
                .map(|window| {
                    let hwnd = window.handle as HWND;
                    (
                        window.handle,
                        window.process_id,
                        unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) },
                        unsafe { IsWindowVisible(hwnd) } != 0,
                    )
                })
                .collect(),
        ))
    }
    pub(super) fn check(&self) -> Result<()> {
        for &(handle, expected_pid, style, visible) in &self.0 {
            let hwnd = handle as HWND;
            let mut pid = 0;
            unsafe {
                GetWindowThreadProcessId(hwnd, &mut pid);
            }
            if pid != expected_pid
                || unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } != style
                || (unsafe { IsWindowVisible(hwnd) } != 0) != visible
            {
                return Err("existing Terminal witness changed; cannot claim unchanged external-window evidence".into());
            }
        }
        println!(
            "PASS: {} preexisting Terminal window style/visibility witnesses unchanged",
            self.0.len()
        );
        Ok(())
    }
}
