use super::{
    super::{Result, observe, setup::Setup, underlay::Underlay},
    input,
    launch::{AdapterLauncher, Shared},
    observations::{check, inspect, quit_restored, require_state, transition},
    render,
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
        transition(&mut controller, Event::InteractionToggle)?;
        require_state(&controller, State::Active)?;
        inspect(&shared, |t| {
            check(t, sink, State::Active)?;
            render::verify(t, sink, false)
        })?;
        transition(&mut controller, Event::RequestPassive)?;
        require_state(&controller, State::Passive)?;
        inspect(&shared, |t| {
            check(t, sink, State::Passive)?;
            render::verify(t, sink, true)?;
            input::click_sink(t, sink)
        })?;
        transition(&mut controller, Event::VisibilityToggle)?;
        require_state(
            &controller,
            State::Hidden {
                return_state: VisibleState::Passive,
            },
        )?;
        inspect(&shared, |t| {
            check(
                t,
                sink,
                State::Hidden {
                    return_state: VisibleState::Passive,
                },
            )
        })?;
        transition(&mut controller, Event::VisibilityToggle)?;
        require_state(&controller, State::Passive)?;
        inspect(&shared, |t| check(t, sink, State::Passive))?;
        println!("ACTIVE_PROOF same session survived Passive/Hidden/Passive");
        transition(&mut controller, Event::InteractionToggle)?;
        require_state(&controller, State::Active)?;
        transition(&mut controller, Event::VisibilityToggle)?;
        require_state(
            &controller,
            State::Hidden {
                return_state: VisibleState::Active,
            },
        )?;
        transition(&mut controller, Event::VisibilityToggle)?;
        require_state(&controller, State::Active)?;
        transition(&mut controller, Event::Quit)?;
        inspect(&shared, quit_restored)?;
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
