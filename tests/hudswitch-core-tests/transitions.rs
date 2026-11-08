use super::*;

#[test]
fn hiding_remembers_both_visible_states_and_active_restore_recaptures_focus() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(
        &controller,
        &world,
        State::Hidden {
            return_state: VisibleState::Active,
        },
    );
    assert_eq!(world.lock().unwrap().foreground, Some(10));
    world.lock().unwrap().foreground = Some(11);
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(&controller, &world, State::Active);
    controller.dispatch(Event::RequestPassive).unwrap();
    assert_eq!(world.lock().unwrap().foreground, Some(11));
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(
        &controller,
        &world,
        State::Hidden {
            return_state: VisibleState::Passive,
        },
    );
    world.lock().unwrap().foreground = Some(12);
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
    assert_eq!(world.lock().unwrap().foreground, Some(12));
}

#[test]
fn interaction_from_hidden_discards_passive_memory() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_view(&controller, &world, State::Active);
}

#[test]
fn passive_requests_are_noops_outside_active() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::RequestPassive).unwrap();
    assert!(world.lock().unwrap().calls.is_empty());
    controller.dispatch(Event::VisibilityToggle).unwrap();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    let status = controller.status();
    world.lock().unwrap().calls.clear();
    controller.dispatch(Event::RequestPassive).unwrap();
    assert_eq!(controller.status(), status);
    assert_eq!(world.lock().unwrap().calls, ["validate"]);
}
