use super::{
    super::{Result, underlay::Underlay},
    launch::Launched,
};
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};
pub(super) fn ensure(target: &Launched) -> Result<()> {
    let mut pid = 0;
    let tid = unsafe { GetWindowThreadProcessId(target.hwnd as _, &mut pid) };
    if tid == 0
        || tid != target.tid
        || pid != target.pid
        || unsafe {
            windows_sys::Win32::System::Threading::GetProcessId(target.owner.as_raw_handle())
        } != target.pid
        || unsafe {
            windows_sys::Win32::System::Threading::WaitForSingleObject(
                target.owner.as_raw_handle(),
                0,
            )
        } != WAIT_TIMEOUT
        || !target.session.alive()
    {
        return Err("Active proof target/session identity changed".into());
    }
    Ok(())
}
pub(super) fn fullscreen(target: &Launched) -> Result<RECT> {
    ensure(target)?;
    let mut rect = unsafe { std::mem::zeroed() };
    if unsafe { GetWindowRect(target.hwnd as _, &mut rect) } == 0 {
        return Err("cannot read target bounds".into());
    }
    if rect.left != 0
        || rect.top != 0
        || rect.right != unsafe { GetSystemMetrics(SM_CXSCREEN) }
        || rect.bottom != unsafe { GetSystemMetrics(SM_CYSCREEN) }
    {
        return Err(format!(
            "target is not primary fullscreen: {},{},{},{}",
            rect.left, rect.top, rect.right, rect.bottom
        )
        .into());
    }
    Ok(rect)
}
pub(super) fn owned_point(target: &Launched, sink: &Underlay, point: POINT) -> Result<()> {
    ensure(target)?;
    sink.ensure_owned()?;
    let hit = unsafe { WindowFromPoint(point) };
    let root = unsafe { GetAncestor(hit, GA_ROOT) };
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(root, &mut pid);
    }
    if !((root == target.hwnd as _ && pid == target.pid)
        || (root == sink.hwnd() && pid == std::process::id()))
    {
        return Err("sample/input point is not on owned target or sink".into());
    }
    Ok(())
}
