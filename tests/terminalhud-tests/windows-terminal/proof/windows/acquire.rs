use super::{
    Result, foreground::Monitor, managed::Managed, session::Session, setup::Setup,
    underlay::Underlay,
};
use hudswitch_core::VisibleState;
use hudswitch_windows::LaunchedWindow;
use std::{
    collections::HashSet,
    thread,
    time::{Duration, Instant},
};
use terminalhud_windows_terminal::{LaunchPlan, native};
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

pub(super) fn acquire(setup: &Setup, sink: &Underlay) -> Result<Managed> {
    if unsafe { GetForegroundWindow() } != sink.hwnd() {
        return Err(
            "owned sink is not foreground; refusing launch instead of bypassing focus policy"
                .into(),
        );
    }
    let before: HashSet<_> = native::windows()?
        .iter()
        .map(|window| window.handle)
        .collect();
    let monitor = Monitor::start()?;
    let before_foreground = unsafe { GetForegroundWindow() };
    if before_foreground != sink.hwnd() {
        return Err("foreground changed before creation; refusing launch".into());
    }
    let plan = LaunchPlan {
        identity: setup.identity.clone(),
        intent: VisibleState::Passive,
        primary_origin: (0, 0),
    };
    let mut session = Session::new(
        setup,
        native::start_with_command(&plan, &setup.terminal, &setup.task, &setup.command)?,
    );
    let proof = session.handshake()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let verified = loop {
        match native::verify(&setup.identity, &proof, &before, &setup.terminal) {
            Ok(window) => break window,
            Err(error) if Instant::now() >= deadline => return Err(error.into()),
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    };
    thread::sleep(Duration::from_millis(250));
    let hwnd = verified.window_handle();
    let foreground_events = monitor.finish()?;
    println!(
        "Terminal PID={} HWND={hwnd} fixture PID={} WT_SESSION={}",
        verified.process_id(),
        proof.shell_pid,
        proof.session
    );
    let final_foreground = unsafe { GetForegroundWindow() };
    println!(
        "FOCUS_EVIDENCE before={} sink={} target={} final={} targetSeen={} events={foreground_events:?}",
        before_foreground as isize,
        sink.hwnd() as isize,
        hwnd,
        final_foreground as isize,
        foreground_events.contains(&hwnd)
    );
    if foreground_events.contains(&hwnd) || final_foreground != sink.hwnd() {
        return Err("Passive creation changed foreground (including transient events); no backend acquisition performed".into());
    }
    println!(
        "PASS: monitored Passive creation preserved owned foreground, {} foreground notifications observed",
        foreground_events.len()
    );
    let (hwnd, process) = verified.into_parts();
    // Correlated nonce/profile/session + live retained ancestry, exact fresh
    // window/executable and task-owned portable path prove this launched target.
    let receipt = unsafe { LaunchedWindow::from_verified_launch(hwnd as _, process) }?;
    Managed::new(receipt, hwnd, session)
}
