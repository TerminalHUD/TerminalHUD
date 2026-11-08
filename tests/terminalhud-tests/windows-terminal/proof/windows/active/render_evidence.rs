//! Acceptance thresholds for the synthetic blue-underlay/black-terminal fixture.
pub(crate) const TOLERANCE: i32 = 6;
pub(crate) fn expected_background(passive: bool) -> [i32; 3] {
    if passive {
        [40, 160, 220]
    } else {
        [26, 104, 143]
    }
}
pub(crate) fn consistent(background: [i32; 3], white_samples: usize, passive: bool) -> bool {
    white_samples >= 10
        && background
            .iter()
            .zip(expected_background(passive))
            .all(|(actual, expected)| (actual - expected).abs() <= TOLERANCE)
}
