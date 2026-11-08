use super::*;

#[test]
fn activation_or_observation_failure_recovers_to_confirmed_passive() {
    for failure in ["present.active", "activate", "observe", "mismatch"] {
        let (mut controller, world) = setup();
        world.lock().unwrap().fail(&[failure]);
        let error = controller.dispatch(Event::InteractionToggle).unwrap_err();
        assert_eq!(error.recovery, Recovery::Passive, "failure at {failure}");
        assert_view(&controller, &world, State::Passive);
        assert_eq!(world.lock().unwrap().foreground, Some(10));
    }
}

#[test]
fn failed_passive_recovery_falls_back_to_hidden_with_passive_memory() {
    let (mut controller, world) = setup();
    world.lock().unwrap().fail(&["activate", "present.passive"]);
    let error = controller.dispatch(Event::InteractionToggle).unwrap_err();
    assert_eq!(error.recovery, Recovery::Hidden);
    assert!(!error.recovery_failures.is_empty());
    assert_view(
        &controller,
        &world,
        State::Hidden {
            return_state: VisibleState::Passive,
        },
    );
}

#[test]
fn unconfirmed_recovery_keeps_ownership_and_snapshot_for_retry_or_quit() {
    let (mut controller, world) = setup();
    world
        .lock()
        .unwrap()
        .fail(&["activate", "present.passive", "present.hidden"]);
    let error = controller.dispatch(Event::InteractionToggle).unwrap_err();
    assert_eq!(error.recovery, Recovery::Unconfirmed);
    assert_eq!(controller.status().availability, Availability::Managed);
    assert_eq!(controller.status().state, None);
    assert!(matches!(
        controller
            .dispatch(Event::InteractionToggle)
            .unwrap_err()
            .failure
            .cause,
        Cause::RecoveryRequired
    ));
    controller.recover().unwrap();
    assert_view(&controller, &world, State::Passive);
    controller.dispatch(Event::Quit).unwrap();
    let world = world.lock().unwrap();
    assert!(world.windows.contains_key(&world.target()));
    assert_eq!(world.windows[&world.target()].style, OTHER_STYLE);
    assert_eq!(controller.status().availability, Availability::Unmanaged);
}
