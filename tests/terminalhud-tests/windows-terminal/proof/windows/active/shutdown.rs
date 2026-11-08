//! Verify owned restoration/session survival before accepting any quit warning.
use super::{
    super::{Result, underlay::Underlay},
    focus_log, identity,
    launch::{Launched, Shared},
    observations::inspect,
};
use hudswitch_core::{Controller, Event};
use hudswitch_windows::WindowsBackend;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
fn quit_restored(target: &Launched) -> Result<()> {
    identity::ensure(target)?;
    let style = unsafe { GetWindowLongPtrW(target.hwnd as _, GWL_EXSTYLE) };
    let visible = unsafe { IsWindowVisible(target.hwnd as _) } != 0;
    let mask = (WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_LAYERED) as isize;
    if style & mask != target.original_style & mask || visible != target.original_visible {
        return Err("quit did not restore original owned style/visibility".into());
    }
    println!("ACTIVE_PROOF quit restored owned styles/visibility and left exact session alive");
    Ok(())
}

pub(super) fn quit<L: hudswitch_windows::Launcher>(
    controller: &mut Controller<WindowsBackend<L>>,
    shared: &Shared,
    sink: &Underlay,
) -> Result<()> {
    let outcome = controller
        .dispatch(Event::Quit)
        .map_err(|error| format!("controller quit failed: {error:?}"))?;
    println!(
        "ACTIVE_COMMAND Quit confirmed {:?} focus_warning={:?}",
        outcome.status, outcome.focus_restore_error
    );
    focus_log::snapshot(sink)?;
    inspect(shared, quit_restored)?;
    if let Some(warning) = outcome.focus_restore_error {
        if unsafe { GetForegroundWindow() } != sink.hwnd() {
            return Err(
                format!("quit foreground warning with saved-sink mismatch: {warning:?}").into(),
            );
        }
        eprintln!(
            "ACTIVE_FOCUS_WARNING {warning:?}; quit restoration and exact saved-sink foreground PASSED"
        );
    }
    Ok(())
}
