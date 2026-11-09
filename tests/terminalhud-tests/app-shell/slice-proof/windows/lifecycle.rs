use super::{
    Command, Result, active, client::Owner, observe, target::Launched, underlay::Underlay,
};
use std::{collections::HashSet, path::Path, thread, time::Duration};
use terminalhud_core::integration::{Availability, State, VisibleState};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
fn check(owner: &Owner, target: &Launched, sink: &Underlay, expected: State) -> Result<()> {
    if owner.call(Command::Status)?.status.state != Some(expected) {
        return Err(format!("unconfirmed state {expected:?}").into());
    }
    active::fullscreen(target)?;
    sink.ensure_owned()?;
    let visible = unsafe { IsWindowVisible(target.hwnd as _) } != 0;
    let style = unsafe { GetWindowLongPtrW(target.hwnd as _, GWL_EXSTYLE) } as u32;
    let foreground = unsafe { GetForegroundWindow() };
    let valid = match expected {
        State::Active => {
            visible
                && foreground == target.hwnd as _
                && style & WS_EX_TRANSPARENT == 0
                && style & WS_EX_TOPMOST != 0
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
            "native {expected:?} mismatch: visible={visible} style={style:#x} foreground={}",
            foreground as isize
        )
        .into());
    }
    if visible {
        active::above_sink(target, sink)?;
    }
    Ok(())
}
pub(super) fn demonstrate(
    owner: &Owner,
    terminal: &Path,
    profile: &terminalhud_windows_terminal::Guid,
    before: &HashSet<isize>,
    sink: &Underlay,
) -> Result<()> {
    let target = Launched::read(owner, terminal, profile, before)?;
    check(owner, &target, sink, State::Active)?;
    for (command, state) in [
        (
            Command::VisibilityToggle,
            State::Hidden {
                return_state: VisibleState::Active,
            },
        ),
        (Command::VisibilityToggle, State::Active),
        (Command::Passive, State::Passive),
        (
            Command::VisibilityToggle,
            State::Hidden {
                return_state: VisibleState::Passive,
            },
        ),
        (Command::VisibilityToggle, State::Passive),
        (Command::InteractionToggle, State::Active),
        (Command::Passive, State::Passive),
    ] {
        owner.call(command)?;
        check(owner, &target, sink, state)?;
    }
    // Close only the verified fresh synthetic session, with no work/user input.
    active::ensure(&target)?;
    if unsafe { PostMessageW(target.hwnd as _, WM_CLOSE, 0, 0) } == 0 {
        return Err("owned target close refused".into());
    }
    observe::wait(|| {
        owner
            .call(Command::Status)
            .is_ok_and(|r| r.status.availability == Availability::Absent)
    })?;
    let no_spawn = owner.call(Command::Passive)?;
    if no_spawn.status.availability != Availability::Absent {
        return Err("absent Passive created a target".into());
    }
    thread::sleep(Duration::from_millis(400));
    if owner.call(Command::Status)?.status.availability != Availability::Absent {
        return Err("lifecycle polling respawned a target".into());
    }
    owner.call(Command::VisibilityToggle)?;
    let replacement = Launched::read(owner, terminal, profile, before)?;
    if replacement.nonce == target.nonce || replacement.session_id == target.session_id {
        return Err("relaunch reused old session identity".into());
    }
    check(owner, &replacement, sink, State::Active)?;
    owner.call(Command::Passive)?;
    check(owner, &replacement, sink, State::Passive)?;
    // Only after every Active check: the click is never used to gain foreground eligibility.
    active::click_sink(&replacement, sink)?;
    owner.call(Command::Quit)?;
    owner.wait_exit()?;
    replacement.restored()?;
    Ok(())
}
