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
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let sample = render_sample::read(target, sink, &rect)?;
        let diagnostic = format!(
            "background={:?} opaque_white_samples={} brightest_rgb={:?} brightest_point={:?} scan_bounds={:?}",
            sample.background,
            sample.white,
            sample.brightest.rgb,
            sample.brightest.point,
            sample.bounds
        );
        if render_evidence::consistent(sample.background, sample.white, passive) {
            println!(
                "ACTIVE_RENDER passive={passive} expected={expected:?} tolerance=6 {diagnostic}"
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("renderer compatibility unproven: passive={passive} expected={expected:?} tolerance=6 {diagnostic}").into());
        }
        thread::sleep(Duration::from_millis(50));
    }
}
