//! Startup-only observation does not imply a full lifecycle proof pass.
mod finish;
mod state;
use super::{
    Completion, Result, client, foreground_setup, monitor::Monitor, target::Launched,
    underlay::Underlay, witness,
};
use crate::observation::{
    Stage,
    decision::{self, Fixture},
};
use std::{collections::HashSet, path::Path};
use terminalhud_core::integration::State;
use terminalhud_windows_terminal::{Guid, native};
pub(super) fn run(manifest: &Path, trace: &Path) -> Result<Completion> {
    if !manifest.is_absolute() || !trace.is_absolute() || trace.exists() {
        return Err("absolute manifest and new private trace path required".into());
    }
    client::require_no_owner()?;
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(manifest)?)?;
    let terminal = Path::new(
        value["terminal"]
            .as_str()
            .ok_or("manifest terminal missing")?,
    );
    let profile = Guid::parse(
        value["profile"]
            .as_str()
            .ok_or("manifest profile missing")?,
    )?;
    let witnesses = witness::Witness::capture()?;
    let before: HashSet<_> = native::windows()?.iter().map(|w| w.handle).collect();
    let sink = Underlay::start()?;
    let mut monitor = Monitor::start()?;
    let result = measure(manifest, terminal, &profile, &before, &sink, &monitor);
    let stopped = monitor.stop();
    let written = monitor.write(trace);
    drop(sink);
    let unchanged = witnesses.check();
    // Always attempt safe unwind/trace/witness checks before selecting outcome.
    let outcome = result?;
    stopped?;
    written?;
    unchanged?;
    Ok(outcome)
}
fn measure(
    manifest: &Path,
    terminal: &Path,
    profile: &Guid,
    before: &HashSet<isize>,
    sink: &Underlay,
    monitor: &Monitor,
) -> Result<Completion> {
    if let Err(error) = foreground_setup::prepare(sink) {
        println!(
            "APP_STARTUP_PRECONDITION_REFUSED {error}; no Terminal launched, no product result"
        );
        return Ok(Completion::PreconditionRefused);
    }
    let sink_id = state::sink_identity(sink)?;
    let start_us = monitor.mark(Stage::Startup)?;
    let (baseline, truncated) = monitor.snapshot()?;
    if truncated
        || baseline
            .last()
            .is_none_or(|s| s.input_tick.is_none() || s.foreground != Some(sink_id))
    {
        println!(
            "APP_STARTUP_PRECONDITION_REFUSED observer baseline unavailable/unstable; no Terminal launched"
        );
        return Ok(Completion::PreconditionRefused);
    }
    let owner = client::Owner::start_observed(manifest)?;
    let target = Launched::read(&owner, terminal, profile, before);
    let native_active = target
        .as_ref()
        .map_err(|e| e.to_string())
        .and_then(|t| state::active_now(t, sink).map_err(|e| e.to_string()));
    let active_reply =
        owner.startup_reply.status.state == Some(State::Active) && owner.startup_failure.is_none();
    let stage = if active_reply {
        Stage::InspectActive
    } else {
        Stage::InspectRecovered
    };
    let assessment = monitor.assess(
        stage,
        sink_id,
        target.as_ref().ok().map(state::identity),
        start_us,
        false,
    );
    match &assessment {
        Ok(value) => println!(
            "STARTUP_OBSERVATION {} {value:?}; unchanged raw ticks do not prove zero input",
            value.label()
        ),
        Err(error) => println!("STARTUP_OBSERVATION INCONCLUSIVE observer unavailable: {error}"),
    }
    println!(
        "STARTUP_RAW reply={} native_active={native_active:?} failure={:?}",
        owner.startup_reply.display(),
        owner.startup_failure
    );
    let measured_pass = active_reply && native_active == Ok(true);
    let report = decision::complete(assessment.as_ref().ok(), measured_pass, |measurement| {
        println!(
            "APP_STARTUP_MEASURED outcome={measurement:?} raw_native_success={measured_pass}; frozen before cleanup, full lifecycle proof unclaimed"
        );
        match finish::finish(&owner, target.as_ref().ok(), monitor, sink_id, start_us) {
            Ok(true) => Fixture::Closed,
            Ok(false) => Fixture::Preserved,
            Err(error) => {
                eprintln!("APP_FIXTURE_CLEANUP_FAILED {error}; startup measurement unchanged");
                Fixture::CleanupFailed
            }
        }
    });
    println!(
        "APP_STARTUP_REPORT measurement={:?} fixture={:?} overall_exit={}; cleanup does not explain or relabel startup",
        report.measurement,
        report.fixture,
        report.exit_code()
    );
    Ok(Completion::Startup(report))
}
