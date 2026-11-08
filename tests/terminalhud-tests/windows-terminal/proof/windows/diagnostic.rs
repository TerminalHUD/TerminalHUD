//! Launch/trace/cleanup only. No backend acquisition, style changes or input.
use super::{
    Result, launch, observe, setup::Setup, show::Show, sources, underlay::Underlay,
    witness::Witness,
};
use std::path::Path;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub(super) fn run(task: &Path, show: Show) -> Result<()> {
    let setup = Setup::load(task)?;
    let sources_before = sources::check()?;
    if unsafe { SetProcessDPIAware() } == 0 {
        return Err("cannot set fixture DPI awareness".into());
    }
    let witness = Witness::capture()?;
    let sink = Underlay::start()?;
    observe::wait(|| unsafe { GetForegroundWindow() } == sink.hwnd())?;
    let result = trace_session(&setup, &sink, show);
    if sources::check()? != sources_before {
        return Err("namespace set changed; evidence invalidated".into());
    }
    witness.check()?;
    result
}
fn trace_session(setup: &Setup, sink: &Underlay, show: Show) -> Result<()> {
    let launched = launch::verified(setup, sink, show)?;
    let hwnd = launched.window.window_handle();
    launched.session.request_exit()?;
    observe::wait(|| unsafe { IsWindow(hwnd as _) } == 0 && !launched.session.alive())?;
    println!(
        "PASS: diagnostic nonce session exited and dedicated target closed; no backend or input used"
    );
    if launched.foreground_changed {
        Err("diagnostic confirmed Passive foreground guard failure; not managing target".into())
    } else {
        println!("PASS: diagnostic launch preserved foreground");
        Ok(())
    }
}
