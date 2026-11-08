//! Read only pixels covered by the two verified owned windows, never screenshots.
use super::Result;
use std::ptr::null_mut;
use windows_sys::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};

pub(super) fn sample(expected_hit: HWND, opacity: u8) -> Result<()> {
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let center = POINT {
        x: width / 2,
        y: height / 2,
    };
    if !owned_hit(center, expected_hit) {
        return Err("renderer sample is outside verified owned window".into());
    }
    let dc = unsafe { GetDC(null_mut()) };
    if dc.is_null() {
        return Err("desktop DC unavailable".into());
    }
    let background = unsafe { GetPixel(dc, center.x, center.y) };
    let mut opaque_text_pixels = 0;
    for y in (100..650.min(height)).step_by(3) {
        for x in (30..500.min(width)).step_by(3) {
            if !owned_hit(POINT { x, y }, expected_hit) {
                continue;
            }
            let pixel = unsafe { GetPixel(dc, x, y) };
            if pixel & 255 > 235 && (pixel >> 8) & 255 > 235 && (pixel >> 16) & 255 > 235 {
                opaque_text_pixels += 1;
            }
        }
    }
    unsafe {
        ReleaseDC(null_mut(), dc);
    }
    let actual = [
        background & 255,
        (background >> 8) & 255,
        (background >> 16) & 255,
    ];
    let expected = [40u32, 160, 220].map(|channel| channel * (100 - u32::from(opacity)) / 100);
    println!(
        "Renderer: opacity={opacity}, blankRGB={actual:?}, expectedRGB={expected:?}, opaque-white samples={opaque_text_pixels}"
    );
    if actual.iter().zip(expected).any(|(a, b)| a.abs_diff(b) > 20) || opaque_text_pixels < 32 {
        return Err("actual background/text rendering does not match HUD appearance".into());
    }
    Ok(())
}

fn owned_hit(point: POINT, expected: HWND) -> bool {
    let hit = unsafe { WindowFromPoint(point) };
    hit == expected || (!hit.is_null() && unsafe { GetAncestor(hit, GA_ROOT) } == expected)
}
