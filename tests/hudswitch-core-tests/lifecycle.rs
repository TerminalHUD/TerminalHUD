use super::*;

#[test]
fn closed_target_can_relaunch_and_old_notifications_cannot_touch_replacement() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    let old = controller.status().target_id.unwrap();
    {
        let mut world = world.lock().unwrap();
        let target = world.target();
        world.windows.remove(&target);
    }
    controller.dispatch(Event::TargetClosed(old)).unwrap();
    assert_eq!(controller.status().availability, Availability::Absent);
    controller.dispatch(Event::VisibilityToggle).unwrap();
    let replacement = controller.status();
    assert_ne!(replacement.target_id, Some(old));
    world.lock().unwrap().calls.clear();
    controller.dispatch(Event::TargetClosed(old)).unwrap();
    controller.dispatch(Event::FocusLost(old)).unwrap();
    assert_eq!(controller.status(), replacement);
    assert!(world.lock().unwrap().calls.is_empty());
}

#[test]
fn invalid_target_is_reacquired_without_mutating_the_stale_handle() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    let old = controller.status().target_id;
    {
        let mut world = world.lock().unwrap();
        let target = world.target();
        world.windows.remove(&target);
        world.calls.clear();
    }
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_ne!(controller.status().target_id, old);
    assert_view(&controller, &world, State::Active);
    assert_eq!(
        &world.lock().unwrap().calls[..4],
        ["validate", "foreground", "acquire.active", "snapshot"]
    );
}

#[test]
fn validation_failure_never_claims_a_confirmed_state_or_mutates_unknown_target() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    {
        let mut world = world.lock().unwrap();
        world.fail(&["validate"]);
        world.calls.clear();
    }
    let error = controller.dispatch(Event::VisibilityToggle).unwrap_err();
    assert_eq!(error.recovery, Recovery::Unconfirmed);
    assert_eq!(controller.status().state, None);
    assert_eq!(world.lock().unwrap().calls, ["validate"]);
    controller.recover().unwrap();
    assert_view(&controller, &world, State::Passive);
}
