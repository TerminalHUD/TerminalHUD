use super::super::{
    client, foreground_setup, lifecycle, monitor::Monitor, startup_observe, underlay::Underlay,
    witness,
};
use super::{Result, Trace};
use std::{collections::HashSet, path::Path};
use terminalhud_windows_terminal::{Guid, native};
pub(in super::super) fn run(manifest: &Path, output: &Path) -> Result<i32> {
    let sidecar = output.with_extension("result.json");
    if !manifest.is_absolute() || !output.is_absolute() || output.exists() || sidecar.exists() {
        return Err("absolute manifest/new trace and sidecar required".into());
    }
    client::require_no_owner()?;
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(manifest)?)?;
    let terminal = Path::new(value["terminal"].as_str().ok_or("terminal missing")?);
    let profile = Guid::parse(value["profile"].as_str().ok_or("profile missing")?)?;
    let witnesses = witness::Witness::capture()?;
    let before: HashSet<_> = native::windows()?.iter().map(|w| w.handle).collect();
    let sink = Underlay::start()?;
    let monitor = Monitor::start()?;
    let sink_id = startup_observe::sink_identity(&sink)?;
    let mut trace = Trace::new(monitor, sink_id, profile.clone());
    let mut owner = None;
    let mut owner_epoch = None;
    let mut launch_attempted = false;
    let measured = (|| -> Result<()> {
        foreground_setup::prepare(&sink)?;
        trace.begin("startup-active", 1)?;
        launch_attempted = true;
        owner = Some(client::Owner::start_observed(manifest)?);
        let retained = owner.as_ref().unwrap();
        owner_epoch = Some(retained.epoch()?);
        if let Some(error) = &retained.startup_failure {
            // Retain only this startup receipt before the mandatory normal Quit.
            // Failure to verify it remains metadata uncertainty, never retarget.
            let target =
                super::super::target::Launched::read(retained, terminal, &profile, &before);
            if let Err(metadata) = &target {
                trace.metadata_error = Some(format!("startup epoch unconfirmed: {metadata}"));
                trace.control.signals.insufficient = true;
            }
            return trace.end(Err(error.clone().into()), target.as_ref().ok(), false);
        }
        lifecycle::demonstrate(
            retained,
            terminal,
            &profile,
            &before,
            &sink,
            Some(&mut trace),
        )
    })();
    if let Err(error) = measured {
        if !trace.control.stopped {
            trace.control.signals.insufficient = true;
            let phase = trace
                .active
                .as_ref()
                .map(|p| p.name)
                .unwrap_or("startup-active");
            trace
                .control
                .interrupt(phase, &format!("observer/ownership boundary: {error}"));
            if trace.active.is_some() {
                let _ = trace.end(Err(error), None, false);
            }
        }
    } else {
        trace.control.finish();
    }
    // Mandatory normal restore is outside frozen scenario measurement. Never
    // follow a later receipt or retry an attempted Quit; retain possible work.
    if let Some(retained) = &owner {
        trace.restore(retained);
    }
    let stopped = trace.monitor.stop();
    let written = trace.monitor.write(output);
    drop(sink);
    let unchanged = witnesses.check();
    let witnesses_ok = unchanged.is_ok();
    if let Err(error) = stopped.and(written).and(unchanged) {
        trace.metadata_error = Some(error.to_string());
    }
    trace.write_report(
        &sidecar,
        manifest,
        owner.as_ref(),
        owner_epoch,
        launch_attempted,
        witnesses_ok,
    )
}
