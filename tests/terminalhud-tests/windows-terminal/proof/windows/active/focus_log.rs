//! Snapshot owned saved-focus identity and actual foreground after a transition.
use super::super::{Result, underlay::Underlay};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub(super) fn snapshot(sink: &Underlay) -> Result<()> {
    sink.ensure_owned()?;
    let mut saved_pid = 0;
    let mut actual_pid = 0;
    let mut class = [0u16; 256];
    let saved_tid = unsafe { GetWindowThreadProcessId(sink.hwnd(), &mut saved_pid) };
    let length = unsafe { GetClassNameW(sink.hwnd(), class.as_mut_ptr(), class.len() as i32) };
    if length <= 0 || saved_pid != std::process::id() || saved_tid == 0 {
        return Err("saved sink identity unavailable for focus observation".into());
    }
    let actual = unsafe { GetForegroundWindow() };
    let actual_tid = unsafe { GetWindowThreadProcessId(actual, &mut actual_pid) };
    println!(
        "ACTIVE_FOCUS saved_hwnd={} saved_pid={saved_pid} saved_tid={saved_tid} saved_class={} actual_hwnd={} actual_pid={actual_pid} actual_tid={actual_tid}",
        sink.hwnd() as isize,
        String::from_utf16_lossy(&class[..length as usize]),
        actual as isize
    );
    Ok(())
}
