use super::{Result, input::click_owned, managed::Managed, observe, render, underlay::Underlay};
use hudswitch_core::{Event, State};
use std::{thread, time::Duration};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub(super) fn exercise(managed: &mut Managed, sink: &Underlay) -> Result<()> {
    let target = managed.hwnd as HWND;
    observe::fullscreen(target)?;
    observe::passive(target)?;
    thread::sleep(Duration::from_millis(700));
    let mut render_failures = Vec::new();
    if let Err(error) = render::sample(sink.hwnd(), 0) {
        render_failures.push(error.to_string());
    }
    match managed.dispatch(Event::InteractionToggle) {
        Ok(_) => {
            if let Some(error) = active_then_hide(managed, sink)? {
                render_failures.push(error);
            }
        }
        Err(error) => {
            println!("OBSERVED LIMIT: Active request refused: {error}");
            if managed.state() != Some(State::Passive) {
                return Err("activation refusal did not recover to Passive".into());
            }
            observe::passive(target)?;
        }
    }
    for _ in 0..5 {
        managed.dispatch(Event::VisibilityToggle)?;
        if unsafe { IsWindowVisible(target) } != 0 || !managed.session.alive() {
            return Err("Hidden did not hide only the window while preserving shell".into());
        }
        managed.dispatch(Event::VisibilityToggle)?;
        observe::passive(target)?;
    }
    let point = POINT {
        x: unsafe { GetSystemMetrics(SM_CXSCREEN) } / 2,
        y: unsafe { GetSystemMetrics(SM_CYSCREEN) } / 2,
    };
    observe::wait(|| unsafe { WindowFromPoint(point) } == sink.hwnd())?;
    let count = sink.clicks()?;
    click_owned(point, sink.hwnd());
    observe::wait(|| sink.clicks().is_ok_and(|value| value == count + 1))?;
    println!(
        "PASS: actual click delivered through Passive Terminal to owned sink; 10 Passive/Hidden transitions preserve shell"
    );
    managed.quit()?;
    if !managed.session.alive() {
        return Err("Quit terminated fixture shell".into());
    }
    println!(
        "PASS: controller Quit restored original owned style bits and left Terminal/shell running"
    );
    managed.session.request_exit()?;
    observe::wait(|| unsafe { IsWindow(target) } == 0 && !managed.session.alive())?;
    println!("PASS: explicit nonce fixture exit closed only the dedicated test session");
    if !render_failures.is_empty() {
        return Err(format!(
            "observed renderer incompatibility: {}",
            render_failures.join("; ")
        )
        .into());
    }
    Ok(())
}

fn active_then_hide(managed: &mut Managed, sink: &Underlay) -> Result<Option<String>> {
    let target = managed.hwnd as HWND;
    if managed.state() != Some(State::Active) || unsafe { GetForegroundWindow() } != target {
        return Err("actual Active focus mismatch".into());
    }
    thread::sleep(Duration::from_millis(700));
    let renderer_failure = render::sample(target, 35)
        .err()
        .map(|error| error.to_string());
    let hidden = managed.dispatch(Event::VisibilityToggle)?;
    if hidden.focus_restore_error.is_some() || unsafe { GetForegroundWindow() } != sink.hwnd() {
        return Err("Hidden(Active) did not restore the owned sink".into());
    }
    println!(
        "PASS: Active focus and Hidden restoration to owned sink; rendering verified={}",
        renderer_failure.is_none()
    );
    match managed.dispatch(Event::VisibilityToggle) {
        Ok(_) => {
            managed.dispatch(Event::RequestPassive)?;
        }
        Err(error) => {
            println!("OBSERVED LIMIT: repeated Active restoration refused: {error}");
            if managed.state() != Some(State::Passive) {
                return Err("refused Active restore was not safe Passive".into());
            }
        }
    }
    observe::passive(target)?;
    Ok(renderer_failure)
}
