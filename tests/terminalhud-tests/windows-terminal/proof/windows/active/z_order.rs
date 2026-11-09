//! Prove relative owned-window stacking; hit testing alone cannot prove visibility.
use super::{
    super::{Result, underlay::Underlay},
    identity,
    launch::Launched,
};
use std::collections::HashSet;
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub(super) fn ensure(target: &Launched, sink: &Underlay) -> Result<()> {
    bands(target, sink)?;
    // Walk toward the top of the actual Z order. Bound the read-only observation
    // even if unrelated windows are concurrently destroyed or reordered.
    let mut current = sink.hwnd();
    let mut seen = HashSet::new();
    for _ in 0..1024 {
        current = unsafe { GetWindow(current, GW_HWNDPREV) };
        if current.is_null() || !seen.insert(current as isize) {
            return Err("capture/input refused: target-above-background Z order not proved (end or concurrent change); owned background may obscure target".into());
        }
        if current == target.hwnd as HWND {
            // This is a non-atomic observation; capture/input also retains its
            // existing identity, foreground, geometry and per-point hit guards.
            return bands(target, sink);
        }
    }
    Err(
        "capture/input refused: target-above-background Z-order observation exceeded 1024 windows"
            .into(),
    )
}

fn bands(target: &Launched, sink: &Underlay) -> Result<()> {
    identity::ensure(target)?;
    sink.ensure_owned()?;
    let target_style = style(target.hwnd as HWND)?;
    let sink_style = style(sink.hwnd())?;
    if target_style & WS_EX_TOPMOST as isize == 0 || sink_style & WS_EX_TOPMOST as isize != 0 {
        return Err(format!(
            "capture/input refused: expected topmost target above non-topmost owned background; target_style={target_style:#x} background_style={sink_style:#x}"
        ).into());
    }
    Ok(())
}

fn style(hwnd: HWND) -> Result<isize> {
    unsafe { SetLastError(0) };
    let value = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    let error = unsafe { GetLastError() };
    if value == 0 && error != 0 {
        return Err(format!("cannot read owned-window Z-order style: Win32 error {error}").into());
    }
    Ok(value)
}
