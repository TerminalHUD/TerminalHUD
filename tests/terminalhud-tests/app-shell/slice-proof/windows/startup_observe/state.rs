use super::super::{Result, active, target::Launched, underlay::Underlay};
use crate::observation::Window;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub(super) fn identity(target: &Launched) -> Window {
    Window {
        hwnd: target.hwnd,
        pid: target.pid,
        tid: target.tid,
    }
}
pub(super) fn sink_identity(sink: &Underlay) -> Result<Window> {
    sink.ensure_owned()?;
    let mut pid = 0;
    let tid = unsafe { GetWindowThreadProcessId(sink.hwnd(), &mut pid) };
    if pid != std::process::id() || tid == 0 {
        return Err("sink identity unavailable".into());
    }
    Ok(Window {
        hwnd: sink.hwnd() as isize,
        pid,
        tid,
    })
}
pub(super) fn active_now(target: &Launched, sink: &Underlay) -> Result<bool> {
    active::ensure(target)?;
    let visible = unsafe { IsWindowVisible(target.hwnd as _) } != 0;
    let style = unsafe { GetWindowLongPtrW(target.hwnd as _, GWL_EXSTYLE) } as u32;
    let foreground = unsafe { GetForegroundWindow() };
    active::ensure(target)?;
    println!(
        "STARTUP_NATIVE hwnd={} pid={} tid={} visible={visible} exstyle={style:#x} foreground={}",
        target.hwnd, target.pid, target.tid, foreground as isize
    );
    if !visible
        || style & WS_EX_TOPMOST == 0
        || style & WS_EX_TRANSPARENT != 0
        || foreground != target.hwnd as _
    {
        return Ok(false);
    }
    active::fullscreen(target)?;
    active::above_sink(target, sink)?;
    Ok(true)
}
