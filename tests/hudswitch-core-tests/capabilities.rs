use super::*;

#[test]
fn unsupported_requests_are_rejected_before_foreground_capture_or_mutations() {
    let (mut controller, world) = setup();
    world.lock().unwrap().capabilities.passive = false;
    let error = controller.dispatch(Event::VisibilityToggle).unwrap_err();
    assert!(matches!(error.failure.cause, Cause::Unsupported));
    assert!(world.lock().unwrap().calls.is_empty());
    world.lock().unwrap().capabilities.hidden = false;
    let error = controller.dispatch(Event::InteractionToggle).unwrap_err();
    assert!(matches!(error.failure.cause, Cause::Unsupported));
    assert!(world.lock().unwrap().calls.is_empty());
}

#[test]
fn launch_and_active_capability_refusal_do_not_create_or_modify_targets() {
    for capabilities in [
        Capabilities {
            launch: false,
            ..Capabilities::ALL
        },
        Capabilities {
            active: false,
            ..Capabilities::ALL
        },
    ] {
        let (mut controller, world) = setup();
        world.lock().unwrap().capabilities = capabilities;
        assert!(matches!(
            controller
                .dispatch(Event::InteractionToggle)
                .unwrap_err()
                .failure
                .cause,
            Cause::Unsupported
        ));
        assert!(world.lock().unwrap().calls.is_empty());
        assert_eq!(controller.status().availability, Availability::Absent);
    }
}
