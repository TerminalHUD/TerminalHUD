//! Fixed owned-region capture, cheap hit testing, and unchanged pure RGB sampling.
use super::{
    super::{Result, underlay::Underlay},
    launch::Launched,
    render_capture,
    render_evidence::Brightest,
    render_ownership::{hit, validate},
    render_region::Region,
};
use std::time::Instant;
use windows_sys::Win32::Foundation::*;
#[derive(Debug)]
pub(super) struct Sample {
    pub(super) background: [i32; 3],
    pub(super) white: usize,
    pub(super) brightest: Brightest,
    pub(super) bounds: (i32, i32, i32, i32),
}
pub(super) fn read(
    target: &Launched,
    sink: &Underlay,
    rect: &RECT,
    passive: bool,
    deadline: Instant,
) -> Result<Sample> {
    let region = Region::new(
        rect.left,
        rect.top,
        ((rect.right - rect.left) / 3).min(512) as usize,
        ((rect.bottom - rect.top) / 2).min(512) as usize,
    )?;
    let background = Region::new(
        rect.left + (rect.right - rect.left) * 3 / 4,
        rect.top + (rect.bottom - rect.top) * 3 / 4,
        1,
        1,
    )?;
    let ownership_started = Instant::now();
    let ownership = (|| {
        validate(target, sink, rect, passive, deadline)?;
        for (x, y) in region.points().chain(background.points()) {
            render_capture::budget(deadline)?;
            hit(target, sink, x, y)?;
        }
        Ok::<_, Box<dyn std::error::Error>>(())
    })();
    println!(
        "ACTIVE_CAPTURE ownership_before_ms={} result_ok={}",
        ownership_started.elapsed().as_millis(),
        ownership.is_ok()
    );
    ownership?;
    let capture_started = Instant::now();
    let images = (|| {
        Ok::<_, Box<dyn std::error::Error>>((
            render_capture::read(region, deadline)?,
            render_capture::read(background, deadline)?,
        ))
    })();
    let capture_duration = capture_started.elapsed();
    println!(
        "ACTIVE_CAPTURE capture_ms={} result_ok={}",
        capture_duration.as_millis(),
        images.is_ok()
    );
    let (pixels, bg) = images?;
    validate(target, sink, rect, passive, deadline)?;
    let scan_started = Instant::now();
    let processed = (|| {
        for (x, y) in region.points().chain(background.points()) {
            render_capture::budget(deadline)?;
            hit(target, sink, x, y)?;
        }
        let summary = pixels.summarize(deadline)?;
        validate(target, sink, rect, passive, deadline)?;
        Ok::<_, Box<dyn std::error::Error>>(Sample {
            background: bg.rgb(background.left(), background.top())?,
            white: summary.white,
            brightest: summary.brightest,
            bounds: (region.left(), region.top(), region.right(), region.bottom()),
        })
    })();
    println!(
        "ACTIVE_CAPTURE scan_ms={} capture_ms={} result_ok={}",
        scan_started.elapsed().as_millis(),
        capture_duration.as_millis(),
        processed.is_ok()
    );
    processed
}
