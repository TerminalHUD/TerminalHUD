//! Read only owned fixture/sink pixels; no whole-desktop screenshot is saved.
use super::{
    super::{Result, underlay::Underlay},
    identity,
    launch::Launched,
    render_evidence,
};
use std::{
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{Foundation::*, Graphics::Gdi::*};
fn rgb(value: u32) -> [i32; 3] {
    [
        (value & 255) as i32,
        ((value >> 8) & 255) as i32,
        ((value >> 16) & 255) as i32,
    ]
}
fn pixel(dc: HDC, target: &Launched, sink: &Underlay, point: POINT) -> Result<[i32; 3]> {
    identity::owned_point(target, sink, point)?;
    let color = unsafe { GetPixel(dc, point.x, point.y) };
    if color == CLR_INVALID {
        return Err("owned pixel read failed".into());
    }
    Ok(rgb(color))
}
pub(super) fn verify(target: &Launched, sink: &Underlay, passive: bool) -> Result<()> {
    let rect = identity::fullscreen(target)?;
    let expected = if passive {
        [40, 160, 220]
    } else {
        [26, 104, 143]
    };
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let dc = unsafe { GetDC(std::ptr::null_mut()) };
        if dc.is_null() {
            return Err("GetDC for owned pixel proof failed".into());
        }
        let result = sample(dc, target, sink, &rect);
        unsafe {
            ReleaseDC(std::ptr::null_mut(), dc);
        }
        let (background, white) = result?;
        if render_evidence::consistent(background, white, passive) {
            println!(
                "ACTIVE_RENDER passive={passive} background={background:?} expected={expected:?} tolerance=6 opaque_white_samples={white}"
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("renderer compatibility unproven: passive={passive} background={background:?} expected={expected:?} tolerance=6 opaque_white_samples={white}").into());
        }
        thread::sleep(Duration::from_millis(50));
    }
}
fn sample(dc: HDC, target: &Launched, sink: &Underlay, rect: &RECT) -> Result<([i32; 3], usize)> {
    let background = pixel(
        dc,
        target,
        sink,
        POINT {
            x: rect.left + (rect.right - rect.left) * 3 / 4,
            y: rect.top + (rect.bottom - rect.top) * 3 / 4,
        },
    )?;
    let mut white = 0;
    for y in (rect.top..rect.top + ((rect.bottom - rect.top) / 2).min(512)).step_by(8) {
        for x in (rect.left..rect.left + ((rect.right - rect.left) / 3).min(512)).step_by(8) {
            if pixel(dc, target, sink, POINT { x, y })?
                .into_iter()
                .all(|v| v >= 245)
            {
                white += 1;
            }
        }
    }
    Ok((background, white))
}
