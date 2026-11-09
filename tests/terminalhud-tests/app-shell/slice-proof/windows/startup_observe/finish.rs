use super::super::{
    Command, Result, active, client::Owner, monitor::Monitor, observe, target::Launched,
};
use crate::observation::{Stage, Window};
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::{
    Foundation::WAIT_TIMEOUT, System::Threading::WaitForSingleObject, UI::WindowsAndMessaging::*,
};
/// Quit belongs to product behavior; closing the fresh fixture is a separate
/// consented test action, conditional on observation through the last guard.
pub(super) fn finish(
    owner: &Owner,
    target: Option<&Launched>,
    monitor: &Monitor,
    sink: Window,
    start_us: u64,
) -> Result<bool> {
    let observation_ready = monitor.mark(Stage::Quit);
    owner.call(Command::Quit)?;
    owner.wait_exit()?;
    let Some(target) = target else {
        println!(
            "FIXTURE_PRESERVED target identity unavailable; normal Quit confirmed, no close attempted"
        );
        return Ok(false);
    };
    target.restored()?;
    if let Err(error) = observation_ready {
        println!(
            "FIXTURE_PRESERVED observer unavailable after confirmed Quit/restoration: {error}"
        );
        return Ok(false);
    }
    let assessment = monitor.assess(
        Stage::Quit,
        sink,
        Some(super::state::identity(target)),
        start_us,
        true,
    )?;
    println!(
        "CLEANUP_OBSERVATION {} {assessment:?}; startup classification is separate",
        assessment.label()
    );
    if assessment.label() != "NO_DISTURBANCE_OBSERVED" {
        println!(
            "FIXTURE_PRESERVED observed activity/unknown foreground/coverage; possible work preserved after normal Quit, disposition pending"
        );
        return Ok(false);
    }
    // No replacement is followed: the same retained process/HWND/thread/session
    // checked before Quit is reverified immediately before ONE ordinary close.
    active::ensure(target)?;
    // Refresh the observation gate after identity checks and immediately before
    // dispatch, without formatting or an activation request in between.
    let latest = monitor.assess(
        Stage::Quit,
        sink,
        Some(super::state::identity(target)),
        start_us,
        true,
    )?;
    if latest.label() != "NO_DISTURBANCE_OBSERVED" {
        println!("FIXTURE_PRESERVED final observation guard became inconclusive; no close sent");
        return Ok(false);
    }
    if unsafe { PostMessageW(target.hwnd as _, WM_CLOSE, 0, 0) } == 0 {
        return Err("fresh fixture normal close refused; no retry".into());
    }
    observe::wait(|| (unsafe { IsWindow(target.hwnd as _) }) == 0)?;
    observe::wait(|| {
        (unsafe { WaitForSingleObject(target.owner.as_raw_handle(), 0) }) != WAIT_TIMEOUT
            && !target.session.alive()
    })?;
    println!(
        "FIXTURE_CLOSED verified fresh disposable session after normal Quit/preservation; no process killed"
    );
    Ok(true)
}
