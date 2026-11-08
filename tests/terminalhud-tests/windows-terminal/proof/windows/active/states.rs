use super::{
    super::{Result, observe, setup::Setup, underlay::Underlay},
    input,
    launch::{AdapterLauncher, Shared},
    observations::{inspect, transition},
    render,
    shutdown::quit,
};
use hudswitch_core::{Controller, Event, State, VisibleState};
use hudswitch_windows::WindowsBackend;
use std::{cell::RefCell, rc::Rc};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub(super) fn run(setup: &Setup, sink: &Underlay) -> Result<()> {
    let shared: Shared = Rc::new(RefCell::new(None));
    let backend = WindowsBackend::new(AdapterLauncher::new(setup, shared.clone()))?;
    let mut controller = Controller::new(backend);
    println!(
        "ACTIVE_BEFORE_LAUNCH foreground={} owned_sink={}",
        unsafe { GetForegroundWindow() } as isize,
        sink.hwnd() as isize
    );
    let result = (|| -> Result<()> {
        transition(
            &mut controller,
            Event::InteractionToggle,
            State::Active,
            &shared,
            sink,
        )?;
        inspect(&shared, |target| render::verify(target, sink, false))?;
        transition(
            &mut controller,
            Event::RequestPassive,
            State::Passive,
            &shared,
            sink,
        )?;
        inspect(&shared, |target| {
            render::verify(target, sink, true)?;
            input::click_sink(target, sink)
        })?;
        transition(
            &mut controller,
            Event::VisibilityToggle,
            State::Hidden {
                return_state: VisibleState::Passive,
            },
            &shared,
            sink,
        )?;
        transition(
            &mut controller,
            Event::VisibilityToggle,
            State::Passive,
            &shared,
            sink,
        )?;
        println!("ACTIVE_PROOF same session survived Passive/Hidden/Passive");
        transition(
            &mut controller,
            Event::InteractionToggle,
            State::Active,
            &shared,
            sink,
        )?;
        transition(
            &mut controller,
            Event::VisibilityToggle,
            State::Hidden {
                return_state: VisibleState::Active,
            },
            &shared,
            sink,
        )?;
        transition(
            &mut controller,
            Event::VisibilityToggle,
            State::Active,
            &shared,
            sink,
        )?;
        quit(&mut controller, &shared, sink)?;
        Ok(())
    })();
    if let Err(ref error) = result {
        eprintln!("ACTIVE_PROOF_FAILED {error}");
        if let Err(error) = controller.dispatch(Event::Quit) {
            eprintln!("ACTIVE_RESTORE_FAILED {error:?}");
        }
    }
    if let Some(target) = shared.borrow_mut().take() {
        target.session.exit()?;
        observe::wait(|| unsafe { IsWindow(target.hwnd as _) } == 0 && !target.session.alive())?;
        println!("ACTIVE_CLEANUP exact nonce fixture exited and target closed");
    }
    result
}
