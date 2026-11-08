//! Strict bounded paint observations after correlated fixture output has flushed.
use super::{
    super::{Result, underlay::Underlay},
    identity,
    launch::Launched,
    render_evidence, render_sample,
};
use std::{
    thread,
    time::{Duration, Instant},
};
pub(super) fn verify(target: &Launched, sink: &Underlay, passive: bool) -> Result<()> {
    target.session.output_flushed()?;
    let rect = identity::fullscreen(target)?;
    println!(
        "ACTIVE_RENDER_BOUNDS display=({},{},{},{}) fixture=foreground-white rows2-5 cols2-25; scan step8 top512 unchanged",
        rect.left, rect.top, rect.right, rect.bottom
    );
    let expected = render_evidence::expected_background(passive);
    let expected_state = if passive { "Passive" } else { "Active" };
    let context = format!(
        "passive={passive} expected_state={expected_state} expected={expected:?} tolerance=6"
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut last_completed = None;
    let mut nonmatching = 0;
    loop {
        let sample = match render_sample::read(target, sink, &rect, passive, deadline) {
            Ok(sample) => sample,
            Err(error) => {
                if let Some(last) = last_completed {
                    return Err(format!(
                        "render sampling stopped: {context} completed_nonmatching_samples={nonmatching} sample_budget_exhausted={} last_completed_sample=[{last}] next_read_error=[{error}]; renderer compatibility unproven",
                        Instant::now() >= deadline
                    ).into());
                }
                return Err(error);
            }
        };
        let diagnostic = format!(
            "background={:?} opaque_white_samples={} brightest_rgb={:?} brightest_point={:?} scan_bounds={:?}",
            sample.background,
            sample.white,
            sample.brightest.rgb,
            sample.brightest.point,
            sample.bounds
        );
        if render_evidence::consistent(sample.background, sample.white, passive) {
            println!("ACTIVE_RENDER {context} {diagnostic}");
            return Ok(());
        }
        nonmatching += 1;
        if nonmatching == 1 {
            eprintln!("ACTIVE_RENDER_FIRST_MISMATCH {context} {diagnostic}");
        }
        if Instant::now() >= deadline {
            return Err(format!("renderer positive-predicate wait exhausted: {context} completed_nonmatching_samples={nonmatching} last_completed_sample=[{diagnostic}]; renderer compatibility unproven").into());
        }
        last_completed = Some(diagnostic);
        thread::sleep(Duration::from_millis(50));
    }
}
