use super::{
    super::{Result, underlay::Underlay},
    focus_log, identity,
    launch::{Launched, Shared},
};
use hudswitch_core::{Controller, Event, State};
use hudswitch_windows::WindowsBackend;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub(super) fn require_state<L: hudswitch_windows::Launcher>(
    controller: &Controller<WindowsBackend<L>>,
    expected: State,
) -> Result<()> {
    if controller.status().state != Some(expected) {
        return Err(format!("unconfirmed requested state {expected:?}").into());
    }
    Ok(())
}
pub(super) fn inspect(
    shared: &Shared,
    operation: impl FnOnce(&Launched) -> Result<()>,
) -> Result<()> {
    let borrowed = shared.borrow();
    operation(
        borrowed
            .as_ref()
            .ok_or("normal Active launch produced no verified target")?,
    )
}
pub(super) fn check(target: &Launched, sink: &Underlay, state: State) -> Result<()> {
    identity::ensure(target)?;
    identity::fullscreen(target)?;
    let visible = unsafe { IsWindowVisible(target.hwnd as _) } != 0;
    let style = unsafe { GetWindowLongPtrW(target.hwnd as _, GWL_EXSTYLE) } as u32;
    let foreground = unsafe { GetForegroundWindow() };
    let valid = match state {
        State::Active => {
            visible
                && foreground == target.hwnd as _
                && style & WS_EX_TOPMOST != 0
                && style & WS_EX_TRANSPARENT == 0
        }
        State::Passive => {
            visible
                && foreground == sink.hwnd()
                && style & (WS_EX_TOPMOST | WS_EX_TRANSPARENT) == WS_EX_TOPMOST | WS_EX_TRANSPARENT
        }
        State::Hidden { .. } => !visible && foreground == sink.hwnd(),
    };
    if !valid {
        return Err(format!(
            "native state mismatch {state:?}: visible={visible} exstyle={style:#x} foreground={}",
            foreground as isize
        )
        .into());
    }
    println!(
        "ACTIVE_STATE {state:?} native confirmed; shell={} session={}",
        target.shell_pid, target.session_id
    );
    Ok(())
}
pub(super) fn transition<L: hudswitch_windows::Launcher>(
    controller: &mut Controller<WindowsBackend<L>>,
    event: Event,
    expected: State,
    shared: &Shared,
    sink: &Underlay,
) -> Result<()> {
    let outcome = controller
        .dispatch(event)
        .map_err(|error| format!("controller transition failed: {error:?}"))?;
    println!(
        "ACTIVE_COMMAND {event:?} confirmed {:?} focus_warning={:?}",
        outcome.status, outcome.focus_restore_error
    );
    focus_log::snapshot(sink)?;
    require_state(controller, expected)?;
    inspect(shared, |target| check(target, sink, expected))?;
    if let Some(warning) = outcome.focus_restore_error {
        eprintln!(
            "ACTIVE_FOCUS_WARNING {warning:?}; strict state and exact saved-sink foreground assertions PASSED after owned release fallback"
        );
    }
    Ok(())
}
