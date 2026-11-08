//! Launch/trace/cleanup only. No backend acquisition, style changes or input.
use super::{
    Result, focus_evidence::Evidence, foreground_setup, launch, observe, setup::Setup, show::Show,
    sources, startup::Presentation, underlay::Underlay, witness::Witness,
};
use std::path::Path;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub(super) fn run(task: &Path, show: Show, presentation: Presentation) -> Result<()> {
    println!(
        "PHASE diagnostic-start showCommand={} presentation={presentation:?}",
        show.value()
    );
    let setup = Setup::load(task)?;
    println!("PHASE setup-validated");
    let sources_before = sources::check()?;
    if unsafe { SetProcessDPIAware() } == 0 {
        return Err("cannot set fixture DPI awareness".into());
    }
    let witness = Witness::capture()?;
    let result = (|| {
        println!("PHASE sink-create");
        let sink = Underlay::start()?;
        foreground_setup::prepare(&sink)?;
        trace_session(&setup, &sink, show, presentation)
    })(); // Drop/join the owned sink before checking external witnesses.
    let namespaces = sources::check().and_then(|after| {
        if after == sources_before {
            println!("PHASE namespace-check unchanged");
            Ok(())
        } else {
            Err("namespace set changed; evidence invalidated".into())
        }
    });
    let witnesses = witness.check();
    for check in [&namespaces, &witnesses] {
        if let Err(error) = check {
            eprintln!("PHASE external-check failed: {error}");
        }
    }
    result?;
    namespaces?;
    witnesses
}
fn trace_session(
    setup: &Setup,
    sink: &Underlay,
    show: Show,
    presentation: Presentation,
) -> Result<()> {
    println!("PHASE launch-observation-start");
    let launched = launch::verified(setup, sink, show, presentation)?;
    let hwnd = launched.window.window_handle();
    println!("PHASE fixture-exit-request");
    launched.session.request_exit()?;
    observe::wait(|| unsafe { IsWindow(hwnd as _) } == 0 && !launched.session.alive())?;
    println!(
        "PASS: diagnostic nonce session exited and dedicated target closed; no backend or input used"
    );
    println!(
        "DIAGNOSTIC_RESULT presentation={presentation:?} evidence={:?}",
        launched.evidence
    );
    match launched.evidence {
        Evidence::TargetActivated { observation_invalid } => Err(format!(
            "diagnostic confirmed target activation; unrelated_observation_invalid={observation_invalid}; not managing target"
        ).into()),
        Evidence::ObservationInvalid => Err(
            "diagnostic observation invalid: non-sink foreground observed; not attributed to Terminal".into()
        ),
        Evidence::Preserved => {
            println!("PASS: diagnostic {presentation:?} preserved foreground throughout observation; not product acceptance");
            Ok(())
        }
    }
}
