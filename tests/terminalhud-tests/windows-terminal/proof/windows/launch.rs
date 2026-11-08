use super::{
    Result, foreground::Monitor, session::Session, setup::Setup, show::Show, underlay::Underlay,
};
use hudswitch_core::VisibleState;
use std::{
    collections::HashSet,
    thread,
    time::{Duration, Instant},
};
use terminalhud_windows_terminal::{LaunchPlan, native};
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
pub(super) struct Launched {
    pub window: native::VerifiedWindow,
    pub session: Session,
    pub foreground_changed: bool,
}
pub(super) fn verified(setup: &Setup, sink: &Underlay, show: Show) -> Result<Launched> {
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
    let mut session = Session::new(setup, super::startup::start(setup, &plan, show)?);
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
    let changed = foreground_events.contains(&hwnd) || final_foreground != sink.hwnd();
    let classified: Vec<_> = foreground_events
        .iter()
        .map(|event| {
            format!(
                "{}:{}",
                event,
                if *event == hwnd {
                    "own-target"
                } else if *event == sink.hwnd() as isize {
                    "own-sink"
                } else {
                    "other"
                }
            )
        })
        .collect();
    println!("CLASSIFIED_FOREGROUND_EVENTS {classified:?}");
    Ok(Launched {
        window: verified,
        session,
        foreground_changed: changed,
    })
}
