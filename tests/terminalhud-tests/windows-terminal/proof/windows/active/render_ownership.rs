//! Retained identity/context checks and cheap owned-root checks around capture.
use super::{
    super::{Result, underlay::Underlay},
    identity,
    launch::Launched,
    render_capture,
};
use std::time::Instant;
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};
pub(super) fn validate(
    target: &Launched,
    sink: &Underlay,
    rect: &RECT,
    passive: bool,
    deadline: Instant,
) -> Result<()> {
    render_capture::budget(deadline)?;
    identity::ensure(target)?;
    sink.ensure_owned()?;
    let current = identity::fullscreen(target)?;
    if (current.left, current.top, current.right, current.bottom)
        != (rect.left, rect.top, rect.right, rect.bottom)
    {
        return Err("owned capture geometry changed".into());
    }
    let expected = if passive {
        sink.hwnd()
    } else {
        target.hwnd as HWND
    };
    if unsafe { GetForegroundWindow() } != expected {
        return Err("owned capture foreground changed".into());
    }
    Ok(())
}
pub(super) fn hit(target: &Launched, sink: &Underlay, x: i32, y: i32) -> Result<()> {
    let root = unsafe { GetAncestor(WindowFromPoint(POINT { x, y }), GA_ROOT) };
    if root != target.hwnd as HWND && root != sink.hwnd() {
        return Err("capture sample point no longer hits an owned window".into());
    }
    Ok(())
}
