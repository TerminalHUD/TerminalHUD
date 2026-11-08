use super::*;

#[test]
fn correlation_requires_nonce_profile_new_window_and_verified_owner_ancestry() {
    let identity = LaunchIdentity {
        nonce: guid(5),
        profile: guid(4),
    };
    let proof = SessionProof {
        nonce: guid(5),
        profile: guid(4),
        session: guid(6),
        shell_pid: 30,
    };
    let candidate = WindowCandidate {
        handle: 11,
        process_id: 20,
        class: "CASCADIA_HOSTING_WINDOW_CLASS".into(),
        title: identity.title(),
        existed_before_launch: false,
    };
    assert_eq!(
        correlate(
            &identity,
            &proof,
            &[30, 20],
            std::slice::from_ref(&candidate)
        )
        .unwrap()
        .handle,
        11
    );
    assert!(correlate(&identity, &proof, &[30], std::slice::from_ref(&candidate)).is_err());
    let mut old = candidate.clone();
    old.existed_before_launch = true;
    assert!(correlate(&identity, &proof, &[30, 20], &[old]).is_err());
    assert!(
        correlate(
            &identity,
            &proof,
            &[30, 20],
            &[candidate.clone(), candidate]
        )
        .is_err()
    );
    let mut wrong = proof;
    wrong.nonce = guid(7);
    assert!(correlate(&identity, &wrong, &[30, 20], &[]).is_err());
}
