use super::*;
#[test]
fn refresh_never_spawns_and_observes_closure_without_touching_stale_target() {
    let (mut controller, world) = setup();
    controller.refresh().unwrap();
    assert!(world.lock().unwrap().calls.is_empty());
    controller.dispatch(Event::InteractionToggle).unwrap();
    {
        let mut world = world.lock().unwrap();
        let target = world.target();
        world.windows.remove(&target);
        world.calls.clear();
    }
    assert_eq!(
        controller.refresh().unwrap().status.availability,
        Availability::Absent
    );
    assert_eq!(world.lock().unwrap().calls, ["validate"]);
    controller.refresh().unwrap();
    controller.dispatch(Event::Quit).unwrap();
    controller.refresh().unwrap();
    assert_eq!(controller.status().availability, Availability::Unmanaged);
}
#[test]
fn healthy_refresh_preserves_saved_focus_and_hidden_memory_without_reapplying() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    world.lock().unwrap().calls.clear();
    controller.refresh().unwrap();
    assert_eq!(world.lock().unwrap().calls, ["validate", "observe"]);
    controller.dispatch(Event::RequestPassive).unwrap();
    assert_eq!(world.lock().unwrap().foreground, Some(10));
    controller.dispatch(Event::VisibilityToggle).unwrap();
    let hidden = controller.status();
    world.lock().unwrap().calls.clear();
    controller.refresh().unwrap();
    assert_eq!(controller.status(), hidden);
    assert_eq!(world.lock().unwrap().calls, ["validate", "observe"]);
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
}
#[test]
fn refresh_focus_loss_preserves_new_user_foreground() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    {
        let mut world = world.lock().unwrap();
        world.foreground = Some(11);
        world.calls.clear();
    }
    controller.refresh().unwrap();
    assert_view(&controller, &world, State::Passive);
    let world = world.lock().unwrap();
    assert_eq!(world.foreground, Some(11));
    assert!(!world.calls.iter().any(|call| call == "restore_focus"));
}
#[test]
fn refresh_validation_failure_retains_original_and_quit_can_restore() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    world.lock().unwrap().fail(&["validate"]);
    assert_eq!(
        controller.refresh().unwrap_err().recovery,
        Recovery::Unconfirmed
    );
    assert_eq!(controller.status().availability, Availability::Managed);
    assert_eq!(controller.status().state, None);
    controller.dispatch(Event::Quit).unwrap();
    assert_eq!(controller.status().availability, Availability::Unmanaged);
}
#[test]
fn refresh_observer_or_invariant_failures_report_recovery_without_losing_snapshot() {
    for failures in [
        vec!["observe"],
        vec!["mismatch"],
        vec!["observe", "present.passive", "present.hidden"],
    ] {
        let (mut controller, world) = setup();
        controller.dispatch(Event::InteractionToggle).unwrap();
        world.lock().unwrap().fail(&failures);
        let error = controller.refresh().unwrap_err();
        if failures.len() == 3 {
            assert_eq!(error.recovery, Recovery::Unconfirmed);
            assert_eq!(controller.status().state, None);
        } else {
            assert_eq!(error.recovery, Recovery::Passive);
            assert_view(&controller, &world, State::Passive);
        }
        controller.dispatch(Event::Quit).unwrap();
        assert_eq!(controller.status().availability, Availability::Unmanaged);
    }
}
