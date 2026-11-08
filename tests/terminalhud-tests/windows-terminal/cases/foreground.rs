#[path = "../proof/windows/focus_evidence.rs"]
mod evidence;
#[path = "../proof/windows/underlay/identity.rs"]
mod identity;
use evidence::{Evidence, classify};

#[test]
fn only_a_sink_only_observation_can_pass() {
    assert_eq!(classify(10, 20, 10, &[]), Evidence::Preserved);
    assert_eq!(classify(10, 20, 10, &[10]), Evidence::Preserved);
    assert_eq!(
        classify(10, 20, 10, &[30, 10]),
        Evidence::ObservationInvalid
    );
    assert_eq!(classify(10, 20, 30, &[]), Evidence::ObservationInvalid);
}
#[test]
fn target_activation_survives_return_to_sink_and_later_interference() {
    assert_eq!(
        classify(10, 20, 10, &[20, 10]),
        Evidence::TargetActivated {
            observation_invalid: false
        }
    );
    assert_eq!(
        classify(10, 20, 20, &[]),
        Evidence::TargetActivated {
            observation_invalid: false
        }
    );
    assert_eq!(
        classify(10, 20, 10, &[20, 30, 10]),
        Evidence::TargetActivated {
            observation_invalid: true
        }
    );
}
#[test]
fn sink_identity_refuses_other_process_thread_class_or_absent_class() {
    let identity = identity::Identity {
        process: 1,
        thread: 2,
        class: vec![65, 66],
    };
    assert!(identity.matches(1, 2, &[65, 66]));
    for (process, thread, class) in [
        (9, 2, vec![65, 66]),
        (1, 9, vec![65, 66]),
        (1, 2, vec![65, 67]),
        (1, 2, vec![]),
    ] {
        assert!(!identity.matches(process, thread, &class));
    }
}
