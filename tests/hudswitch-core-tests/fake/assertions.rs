use super::*;

pub(crate) fn assert_view(controller: &Controller<Fake>, world: &Arc<Mutex<World>>, state: State) {
    assert_eq!(controller.status().availability, Availability::Managed);
    assert_eq!(controller.status().state, Some(state));
    let observation = world.lock().unwrap().observation();
    match state {
        State::Active => {
            assert!(observation.visible && observation.topmost && observation.focused);
            assert!(!observation.click_through);
        }
        State::Passive => {
            assert!(observation.visible && observation.topmost && observation.click_through);
            assert!(!observation.focused);
        }
        State::Hidden { .. } => assert!(!observation.visible && !observation.focused),
    }
}
