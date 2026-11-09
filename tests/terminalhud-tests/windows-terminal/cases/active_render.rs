use super::render_evidence as evidence;
#[test]
fn renderer_proof_requires_both_expected_background_and_opaque_fixture_samples() {
    assert!(evidence::consistent([26, 104, 143], 10, false));
    assert!(evidence::consistent([40, 160, 220], 10, true));
    assert!(!evidence::consistent([40, 160, 220], 10, false));
    assert!(!evidence::consistent([26, 104, 143], 10, true));
    assert!(!evidence::consistent([26, 104, 143], 0, false));
    assert!(!evidence::consistent([40, 160, 220], 9, true));
    assert!(!evidence::consistent([0, 0, 0], 100, true));
}

#[test]
fn brightness_summary_reports_one_actual_pixel_not_independent_channel_maxima() {
    let mut brightest = evidence::Brightest {
        rgb: [0, 0, 0],
        point: (0, 0),
    };
    brightest.consider([255, 255, 0], (1, 1));
    brightest.consider([243, 243, 243], (2, 2));
    brightest.consider([255, 0, 255], (3, 3));
    assert_eq!(
        brightest,
        evidence::Brightest {
            rgb: [243, 243, 243],
            point: (2, 2)
        }
    );
}
