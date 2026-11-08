mod identity;
mod input;
mod input_recovery;
mod launch;
mod observations;
mod render;
mod render_evidence;
mod session;
mod states;
use super::{
    Result, foreground_setup, setup::Setup, sources, underlay::Underlay, witness::Witness,
};
use std::path::Path;
use windows_sys::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;
pub(super) fn run(task: &Path) -> Result<()> {
    let setup = Setup::load(task)?;
    let before = sources::check()?;
    if unsafe { SetProcessDPIAware() } == 0 {
        return Err("cannot set Active proof DPI awareness".into());
    }
    let witness = Witness::capture()?;
    let result = (|| {
        let sink = Underlay::start()?;
        foreground_setup::prepare(&sink)?;
        states::run(&setup, &sink)
    })();
    let namespaces = sources::check().and_then(|after| {
        if after == before {
            Ok(())
        } else {
            Err("Active proof namespace set changed".into())
        }
    });
    let witnesses = witness.check();
    for check in [&namespaces, &witnesses] {
        if let Err(error) = check {
            eprintln!("ACTIVE_EXTERNAL_CHECK_FAILED {error}");
        }
    }
    result?;
    namespaces?;
    witnesses
}
