use super::render_evidence;
#[path = "../proof/windows/active/render_pixels.rs"]
mod render_pixels;
#[path = "../proof/windows/active/render_region.rs"]
mod render_region;
use render_pixels::Pixels;
use render_region::Region;
use std::time::{Duration, Instant};
#[test]
fn top_down_bgra_stride_and_screen_coordinates_preserve_actual_pixels() {
    let region = Region::new(10, 20, 2, 2).unwrap();
    let bytes = vec![
        3, 2, 1, 0, 6, 5, 4, 0, 99, 99, 99, 99, 9, 8, 7, 0, 12, 11, 10, 0, 88, 88, 88, 88,
    ];
    let pixels = Pixels::new(region, 12, bytes).unwrap();
    assert_eq!(pixels.rgb(10, 20).unwrap(), [1, 2, 3]);
    assert_eq!(pixels.rgb(11, 21).unwrap(), [10, 11, 12]);
    assert!(pixels.rgb(9, 20).is_err());
    assert!(pixels.rgb(12, 20).is_err());
    assert!(pixels.rgb(10, 22).is_err());
}
#[test]
fn capture_bounds_stride_length_and_overflow_are_refused_before_native_use() {
    for (x, y, w, h) in [
        (0, 0, 0, 1),
        (0, 0, 513, 1),
        (0, 0, 1, usize::MAX),
        (i32::MAX, 0, 1, 1),
        (0, i32::MAX, 1, 1),
    ] {
        assert!(Region::new(x, y, w, h).is_err());
    }
    let region = Region::new(0, 0, 2, 2).unwrap();
    for (stride, len) in [(4, 8), (9, 18), (8, 15), (8, 17), (usize::MAX, 0)] {
        assert!(Pixels::new(region, stride, vec![0; len]).is_err());
    }
}
#[test]
fn unchanged_step8_sampling_excludes_padding_and_checks_budget() {
    let region = Region::new(-10, 20, 17, 9).unwrap();
    assert_eq!(
        region.points().collect::<Vec<_>>(),
        vec![(-10, 20), (-2, 20), (6, 20), (-10, 28), (-2, 28), (6, 28)]
    );
    let pixels = Pixels::new(region, 68, vec![255; 68 * 9]).unwrap();
    let summary = pixels
        .summarize(Instant::now() + Duration::from_secs(1))
        .unwrap();
    assert_eq!(summary.white, 6);
    assert_eq!(summary.brightest.rgb, [255, 255, 255]);
    assert!(
        pixels
            .summarize(Instant::now() - Duration::from_millis(1))
            .is_err()
    );
    assert_eq!(region.left(), -10);
    assert_eq!(region.top(), 20);
    assert_eq!(region.width(), 17);
    assert_eq!(region.height(), 9);
    assert_eq!(render_evidence::expected_background(true), [40, 160, 220]);
    assert!(!render_evidence::consistent(
        [40, 160, 220],
        summary.white,
        true
    ));
}
