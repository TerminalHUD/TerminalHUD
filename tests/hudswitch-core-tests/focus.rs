use super::*;

#[test]
fn hiding_a_passive_target_never_restores_a_stale_foreground() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    controller.dispatch(Event::RequestPassive).unwrap();
    world.lock().unwrap().foreground = Some(12);
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_eq!(world.lock().unwrap().foreground, Some(12));
}

#[test]
fn focus_loss_preserves_the_window_the_user_selected() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    let id = controller.status().target_id.unwrap();
    {
        let mut world = world.lock().unwrap();
        world.foreground = Some(11);
        world.calls.clear();
    }
    controller.dispatch(Event::FocusLost(id)).unwrap();
    assert_view(&controller, &world, State::Passive);
    let world = world.lock().unwrap();
    assert_eq!(world.foreground, Some(11));
    assert!(!world.calls.iter().any(|call| call == "restore_focus"));
}

#[test]
fn unavailable_saved_focus_is_a_warning_and_never_retried() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    {
        let mut world = world.lock().unwrap();
        world.fail(&["restore_focus"]);
        world.calls.clear();
    }
    let outcome = controller.dispatch(Event::RequestPassive).unwrap();
    assert!(outcome.focus_restore_error.is_some());
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
