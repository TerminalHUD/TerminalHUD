#[path = "../proof/windows/active/input_recovery.rs"]
mod recovery;

#[test]
fn every_partial_prefix_releases_only_the_probes_unmatched_button_down() {
    for inserted in 0..=3 {
        let mut events = ["move", "left-down", "left-up"][..inserted as usize].to_vec();
        let result = recovery::complete(inserted, || {
            events.push("left-up");
            true
        });
        assert_eq!(result.is_ok(), inserted == 3);
        let downs = events.iter().filter(|event| **event == "left-down").count();
        let ups = events.iter().filter(|event| **event == "left-up").count();
        assert_eq!(downs, ups, "unbalanced inserted prefix {inserted}");
        assert_eq!(
            events.len(),
            if inserted == 2 { 3 } else { inserted as usize }
        );
    }
}

#[test]
fn failed_compensation_is_reported_once_and_success_does_not_release_again() {
    let mut releases = 0;
    let error = recovery::complete(2, || {
        releases += 1;
        false
    })
    .unwrap_err();
    assert!(error.contains("compensating LEFTUP insertion failed"));
    assert_eq!(releases, 1);
    assert!(recovery::complete(3, || panic!("complete click must not release again")).is_ok());
}

#[test]
fn any_held_mouse_button_refuses_the_proof_but_toggle_bits_are_not_presses() {
    assert!(recovery::neutral([0; 5]));
    assert!(recovery::neutral([1; 5]));
    for index in 0..5 {
        let mut states = [0; 5];
        states[index] = i16::MIN;
        assert!(!recovery::neutral(states));
    }
}
