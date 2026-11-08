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

/// Select one real pixel by its weakest channel; never synthesize channel maxima.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Brightest {
    pub(crate) rgb: [i32; 3],
    pub(crate) point: (i32, i32),
}
impl Brightest {
    pub(crate) fn consider(&mut self, rgb: [i32; 3], point: (i32, i32)) {
        if rgb.into_iter().min() > self.rgb.into_iter().min() {
            *self = Self { rgb, point };
        }
    }
}
