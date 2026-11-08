use super::*;

#[test]
fn absent_visibility_launches_without_activation_or_foreground_theft() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
    let world = world.lock().unwrap();
    assert_eq!(world.foreground, Some(10));
    assert_eq!(
        world.calls,
        [
            "acquire.passive",
            "snapshot",
            "present.passive",
            "release_focus",
            "observe"
        ]
    );
}

#[test]
fn active_launch_saves_the_external_foreground_before_acquisition() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_view(&controller, &world, State::Active);
    {
        let world = world.lock().unwrap();
        assert_eq!(
            &world.calls[..3],
            ["foreground", "acquire.active", "snapshot"]
        );
    }
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
    assert_eq!(world.lock().unwrap().foreground, Some(10));
}

#[test]
fn acquisition_and_foreground_errors_do_not_create_managed_ownership() {
    for operation in ["foreground", "acquire.active"] {
        let (mut controller, world) = setup();
        world.lock().unwrap().fail(&[operation]);
        let error = controller.dispatch(Event::InteractionToggle).unwrap_err();
        assert_eq!(error.recovery, Recovery::NotNeeded);
        assert_eq!(controller.status().availability, Availability::Absent);
        assert!(world.lock().unwrap().windows.is_empty());
    }
}
