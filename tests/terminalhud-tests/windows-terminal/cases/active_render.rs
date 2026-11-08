#[path = "../proof/windows/active/render_evidence.rs"]
mod evidence;
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
