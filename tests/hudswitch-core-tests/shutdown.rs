use super::*;

#[test]
fn quit_restores_only_owned_styles_and_leaves_even_hidden_target_open() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    {
        let mut world = world.lock().unwrap();
        let target = world.target();
        world.windows.get_mut(&target).unwrap().style |= 8;
    }
    controller.dispatch(Event::Quit).unwrap();
    let world = world.lock().unwrap();
    let window = world.windows[&world.target()];
    assert!(window.visible);
    assert_eq!(window.style, OTHER_STYLE | 8);
    assert_eq!(controller.status().availability, Availability::Unmanaged);
    assert!(matches!(
        controller
            .dispatch(Event::InteractionToggle)
            .unwrap_err()
            .failure
            .cause,
        Cause::Stopped
    ));
}

#[test]
fn quit_restore_failure_retains_snapshot_and_can_retry_without_false_detach() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    world
        .lock()
        .unwrap()
        .fail(&["restore_original", "present.passive", "present.hidden"]);
    let error = controller.dispatch(Event::Quit).unwrap_err();
    assert_eq!(error.failure.stage, Stage::RestoreOriginal);
    assert_eq!(error.recovery, Recovery::Unconfirmed);
    assert_eq!(controller.status().availability, Availability::Managed);
    assert_eq!(controller.status().state, None);
    controller.dispatch(Event::Quit).unwrap();
    assert_eq!(controller.status().availability, Availability::Unmanaged);
    let world = world.lock().unwrap();
    assert_eq!(world.windows[&world.target()].style, OTHER_STYLE);
}
