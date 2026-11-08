//! Bounded owned-region pixel capture and failure metadata, without screenshots.
use super::{
    super::{Result, underlay::Underlay},
    identity,
    launch::Launched,
    render_evidence::Brightest,
};
use windows_sys::Win32::{Foundation::*, Graphics::Gdi::*};
#[derive(Debug)]
pub(super) struct Sample {
    pub(super) background: [i32; 3],
    pub(super) white: usize,
    pub(super) brightest: Brightest,
    pub(super) bounds: (i32, i32, i32, i32),
}
pub(super) fn read(target: &Launched, sink: &Underlay, rect: &RECT) -> Result<Sample> {
    let dc = unsafe { GetDC(std::ptr::null_mut()) };
    if dc.is_null() {
        return Err("GetDC for owned pixel proof failed".into());
    }
    let result = sample(dc, target, sink, rect);
    unsafe {
        ReleaseDC(std::ptr::null_mut(), dc);
    }
    result
}
fn pixel(dc: HDC, target: &Launched, sink: &Underlay, point: POINT) -> Result<[i32; 3]> {
    identity::owned_point(target, sink, point)?;
    let value = unsafe { GetPixel(dc, point.x, point.y) };
    if value == CLR_INVALID {
        return Err("owned pixel read failed".into());
    }
    Ok([
        (value & 255) as i32,
        ((value >> 8) & 255) as i32,
        ((value >> 16) & 255) as i32,
    ])
}
fn sample(dc: HDC, target: &Launched, sink: &Underlay, rect: &RECT) -> Result<Sample> {
    let background = pixel(
        dc,
        target,
        sink,
        POINT {
            x: rect.left + (rect.right - rect.left) * 3 / 4,
            y: rect.top + (rect.bottom - rect.top) * 3 / 4,
        },
    )?;
    let bounds = (
        rect.left,
        rect.top,
        rect.left + ((rect.right - rect.left) / 3).min(512),
        rect.top + ((rect.bottom - rect.top) / 2).min(512),
    );
    let mut result = Sample {
        background,
        white: 0,
        brightest: Brightest {
            rgb: [0, 0, 0],
            point: (bounds.0, bounds.1),
        },
        bounds,
    };
    for y in (bounds.1..bounds.3).step_by(8) {
        for x in (bounds.0..bounds.2).step_by(8) {
            let rgb = pixel(dc, target, sink, POINT { x, y })?;
            result.brightest.consider(rgb, (x, y));
            if rgb.into_iter().all(|v| v >= 245) {
                result.white += 1;
            }
        }
    }
    Ok(result)
}
