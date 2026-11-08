use super::{Result, acquire, observe, setup::Setup, sources, transitions, underlay::Underlay};
use std::path::Path;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub(super) fn run(task: &Path) -> Result<()> {
    let setup = Setup::load(task)?;
    let sources_before = sources::check()?;
    if unsafe { SetProcessDPIAware() } == 0 {
        return Err("cannot set fixture process DPI awareness".into());
    }
    let witness = super::witness::Witness::capture()?;
    let sink = Underlay::start()?;
    observe::wait(|| unsafe { GetForegroundWindow() } == sink.hwnd())?;
    let mut managed = acquire::acquire(&setup, &sink)?;
    let result = transitions::exercise(&mut managed, &sink);
    if sources::check()? != sources_before {
        return Err("namespace set changed; native evidence invalidated".into());
    }
    witness.check()?;
    result
}
