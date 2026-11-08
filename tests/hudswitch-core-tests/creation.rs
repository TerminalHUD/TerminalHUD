use super::*;

fn active_creation() -> (Controller<fake::Fake>, Arc<Mutex<fake::World>>) {
    let (_, world) = setup();
    let controller =
        Controller::with_creation_state(fake::Fake(world.clone()), VisibleState::Active);
    (controller, world)
}

#[test]
fn creation_policy_captures_focus_before_visibility_launch_and_restores_it() {
    let (mut controller, world) = active_creation();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(&controller, &world, State::Active);
    assert_eq!(
        &world.lock().unwrap().calls[..3],
        ["foreground", "acquire.active", "snapshot"]
    );
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
    assert_eq!(world.lock().unwrap().foreground, Some(10));
}

#[test]
fn closed_target_notifications_do_not_change_replacement_creation_policy() {
    let (mut controller, world) = active_creation();
    controller.dispatch(Event::InteractionToggle).unwrap();
    let old = controller.status().target_id.unwrap();
    {
        let mut world = world.lock().unwrap();
        let target = world.target();
        world.windows.remove(&target);
    }
    controller.dispatch(Event::TargetClosed(old)).unwrap();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(&controller, &world, State::Active);
    assert_ne!(controller.status().target_id, Some(old));
    let replacement = controller.status();
    controller.dispatch(Event::TargetClosed(old)).unwrap();
    assert_eq!(controller.status(), replacement);
}

#[test]
fn selected_creation_state_is_capability_checked_before_capture_or_launch() {
    let (mut controller, world) = active_creation();
    world.lock().unwrap().capabilities.active = false;
    assert!(matches!(
        controller
            .dispatch(Event::VisibilityToggle)
            .unwrap_err()
            .failure
            .cause,
        Cause::Unsupported
    ));
    assert!(world.lock().unwrap().calls.is_empty());
}

#[test]
fn explicit_passive_creation_policy_preserves_foreground() {
    let (_, world) = setup();
    let mut controller =
        Controller::with_creation_state(fake::Fake(world.clone()), VisibleState::Passive);
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
    assert_eq!(
        &world.lock().unwrap().calls[..2],
        ["acquire.passive", "snapshot"]
    );
    assert_eq!(world.lock().unwrap().foreground, Some(10));
}
