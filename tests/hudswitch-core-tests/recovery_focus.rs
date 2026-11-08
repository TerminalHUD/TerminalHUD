use super::*;

#[test]
fn fallback_does_not_retry_an_unavailable_saved_foreground() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    {
        let mut world = world.lock().unwrap();
        world.calls.clear();
        world.fail(&["restore_focus", "release_focus"]);
    }
    let error = controller.dispatch(Event::RequestPassive).unwrap_err();
    assert_eq!(error.recovery, Recovery::Passive);
    assert_view(&controller, &world, State::Passive);
    let world = world.lock().unwrap();
    assert_eq!(world.foreground, None);
    assert_eq!(
        world
            .calls
            .iter()
            .filter(|call| *call == "restore_focus")
            .count(),
        1
    );
}

#[test]
fn explicit_recovery_preserves_a_new_foreground_after_an_unconfirmed_failure() {
    let (mut controller, world) = setup();
    world
        .lock()
        .unwrap()
        .fail(&["activate", "present.passive", "present.hidden"]);
    controller.dispatch(Event::InteractionToggle).unwrap_err();
    world.lock().unwrap().foreground = Some(12);
    controller.recover().unwrap();
    assert_view(&controller, &world, State::Passive);
    assert_eq!(world.lock().unwrap().foreground, Some(12));
}
