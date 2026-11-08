#[path = "../proof/windows/underlay/identity.rs"]
mod identity;

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
