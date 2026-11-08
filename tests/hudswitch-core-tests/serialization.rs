use super::*;

#[test]
fn overlapping_host_events_complete_whole_transitions_before_the_next_begins() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    world.lock().unwrap().calls.clear();
    let controller = Arc::new(Mutex::new(controller));
    let start = Arc::new(Barrier::new(3));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let controller = controller.clone();
            let start = start.clone();
            thread::spawn(move || {
                start.wait();
                controller
                    .lock()
                    .unwrap()
                    .dispatch(Event::InteractionToggle)
                    .unwrap();
            })
        })
        .collect();
    start.wait();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_view(&controller.lock().unwrap(), &world, State::Passive);
    let world = world.lock().unwrap();
    assert_eq!(
        world.calls,
        [
            "validate",
            "foreground",
            "present.active",
            "activate",
            "observe",
            "validate",
            "present.passive",
            "restore_focus",
            "release_focus",
            "observe"
        ]
    );
}
