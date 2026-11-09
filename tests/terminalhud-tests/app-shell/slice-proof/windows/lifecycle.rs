use super::{
    Command, Result, active,
    client::Owner,
    full_observe::{self, Trace},
    observe,
    target::Launched,
    underlay::Underlay,
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
    mut observation: Option<&mut Trace>,
) -> Result<()> {
    let target = Launched::read(owner, terminal, profile, before)?;
    full_observe::end(
        &mut observation,
        check(owner, &target, sink, State::Active),
        Some(&target),
        true,
    )?;
    for (name, command, state) in [
        (
            "hide-active",
            Command::VisibilityToggle,
            State::Hidden {
                return_state: VisibleState::Active,
            },
        ),
        ("restore-active", Command::VisibilityToggle, State::Active),
        ("passive", Command::Passive, State::Passive),
        (
            "hide-passive",
            Command::VisibilityToggle,
            State::Hidden {
                return_state: VisibleState::Passive,
            },
        ),
        ("restore-passive", Command::VisibilityToggle, State::Passive),
        ("activate", Command::InteractionToggle, State::Active),
        ("passive-before-close", Command::Passive, State::Passive),
    ] {
        full_observe::begin(&mut observation, name, 1)?;
        let result = owner
            .call(command)
            .and_then(|_| check(owner, &target, sink, state));
        full_observe::end(
            &mut observation,
            result,
            Some(&target),
            state == State::Active,
        )?;
    }
    // Original sequence reaches Passive here. The observer may prevent closing
    // on uncertainty, but never changes the original close/assertion ordering.
    full_observe::begin(&mut observation, "close-first", 1)?;
    if let Some(trace) = &mut observation {
        trace.close_gate(&target, check(owner, &target, sink, State::Passive))?;
    }
    let closed = close_first(owner, &target);
    if let Some(trace) = &mut observation {
        trace.after_close(&target, closed)?;
    } else {
        closed?;
    }
    full_observe::begin(&mut observation, "absent-passive", 1)?;
    let no_spawn = owner.call(Command::Passive).and_then(|reply| {
        if reply.status.availability == Availability::Absent {
            Ok(())
        } else {
            Err("absent Passive created a target".into())
        }
    });
    full_observe::end(&mut observation, no_spawn, None, false)?;
    full_observe::begin(&mut observation, "absent-poll", 1)?;
    thread::sleep(Duration::from_millis(400));
    let absent = owner.call(Command::Status).and_then(|reply| {
        if reply.status.availability == Availability::Absent {
            Ok(())
        } else {
            Err("lifecycle polling respawned a target".into())
        }
    });
    full_observe::end(&mut observation, absent, None, false)?;
    full_observe::begin(&mut observation, "relaunch-active", 2)?;
    if let Some(trace) = &mut observation {
        trace.relaunch_attempt();
    }
    let launched = (|| -> Result<Launched> {
        owner.call(Command::VisibilityToggle)?;
        let replacement = Launched::read(owner, terminal, profile, before)?;
        if replacement.nonce == target.nonce || replacement.session_id == target.session_id {
            return Err("relaunch reused old session identity".into());
        }
        if let Some(trace) = &mut observation {
            trace.remember(&replacement)?;
        }
        check(owner, &replacement, sink, State::Active)?;
        Ok(replacement)
    })();
    let replacement = match launched {
        Ok(target) => target,
        Err(error) => return full_observe::end(&mut observation, Err(error), None, true),
    };
    full_observe::end(&mut observation, Ok(()), Some(&replacement), true)?;
    full_observe::begin(&mut observation, "final-passive", 2)?;
    let passive = owner
        .call(Command::Passive)
        .and_then(|_| check(owner, &replacement, sink, State::Passive));
    full_observe::end(&mut observation, passive, Some(&replacement), false)?;
    // Only after every Active check: the click is never used to gain foreground eligibility.
    full_observe::begin(&mut observation, "planned-click", 2)?;
    let click_begin = observation.as_mut().map(|trace| trace.click_begin());
    let clicked = active::click_sink(&replacement, sink);
    if let (Some(trace), Some(begin)) = (&mut observation, click_begin) {
        trace.click_end(begin, clicked.is_ok());
    }
    full_observe::end(&mut observation, clicked, Some(&replacement), false)?;
    full_observe::begin(&mut observation, "quit-restore", 2)?;
    let restored = owner
        .call(Command::Quit)
        .and_then(|_| owner.wait_exit())
        .and_then(|_| replacement.restored());
    full_observe::end(&mut observation, restored, Some(&replacement), false)?;
    Ok(())
}
fn close_first(owner: &Owner, target: &Launched) -> Result<()> {
    active::ensure(target)?;
    if unsafe { PostMessageW(target.hwnd as _, WM_CLOSE, 0, 0) } == 0 {
        return Err("owned target close refused".into());
    }
    observe::wait(|| {
        owner
            .call(Command::Status)
            .is_ok_and(|r| r.status.availability == Availability::Absent)
    })
}
