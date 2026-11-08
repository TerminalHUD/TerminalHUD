use hudswitch_core::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

const TOPMOST: u32 = 1;
const CLICK_THROUGH: u32 = 2;
const OWNED: u32 = TOPMOST | CLICK_THROUGH;
const OTHER_STYLE: u32 = 4;

#[derive(Clone, Copy)]
struct Window {
    visible: bool,
    style: u32,
}

struct World {
    windows: HashMap<u64, Window>,
    foreground: Option<u64>,
    next_target: u64,
    calls: Vec<String>,
    failures: VecDeque<&'static str>,
    capabilities: Capabilities,
}

impl World {
    fn step(&mut self, operation: &str) -> bool {
        self.calls.push(operation.to_owned());
        if self.failures.front().copied() == Some(operation) {
            self.failures.pop_front();
            true
        } else {
            false
        }
    }

    fn target(&self) -> u64 {
        self.next_target
    }

    fn observation(&self) -> Observation {
        let target = self.target();
        let window = self.windows[&target];
        Observation {
            visible: window.visible,
            topmost: window.style & TOPMOST != 0,
            click_through: window.style & CLICK_THROUGH != 0,
            focused: self.foreground == Some(target),
        }
    }

    fn fail(&mut self, operations: &[&'static str]) {
        self.failures.extend(operations);
    }
}

#[derive(Clone)]
struct Fake(Arc<Mutex<World>>);

struct Original {
    owned_style: u32,
    visible: bool,
}

impl Backend for Fake {
    type Target = u64;
    type Focus = u64;
    type Original = Original;
    type Error = &'static str;

    fn capabilities(&self) -> Capabilities {
        self.0.lock().unwrap().capabilities
    }

    fn acquire(&mut self, intent: VisibleState) -> Result<Acquired<u64, Original>, Self::Error> {
        let mut world = self.0.lock().unwrap();
        let operation = match intent {
            VisibleState::Active => "acquire.active",
            VisibleState::Passive => "acquire.passive",
        };
        if world.step(operation) {
            return Err("acquire failed without taking ownership");
        }
        world.next_target += 1;
        let target = world.target();
        let window = Window {
            visible: true,
            style: OTHER_STYLE,
        };
        let original = Original {
            owned_style: window.style & OWNED,
            visible: window.visible,
        };
        world.calls.push("snapshot".into());
        world.windows.insert(target, window);
        if intent == VisibleState::Active {
            world.foreground = Some(target);
        }
        Ok(Acquired { target, original })
    }

    fn is_valid(&mut self, target: &u64) -> Result<bool, Self::Error> {
        let mut world = self.0.lock().unwrap();
        if world.step("validate") {
            return Err("identity cannot be validated");
        }
        Ok(world.windows.contains_key(target))
    }

    fn foreground(&mut self) -> Result<Option<u64>, Self::Error> {
        let mut world = self.0.lock().unwrap();
        if world.step("foreground") {
            return Err("foreground unavailable");
        }
        Ok(world.foreground)
    }

    fn present(&mut self, target: &u64, presentation: Presentation) -> Result<(), Self::Error> {
        let mut world = self.0.lock().unwrap();
        let operation = match presentation {
            Presentation::Active => "present.active",
            Presentation::Passive => "present.passive",
            Presentation::Hidden => "present.hidden",
        };
        let fail = world.step(operation);
        // Failures may occur after a partial mutation, as real backend calls can.
        let window = world.windows.get_mut(target).ok_or("target closed")?;
        match presentation {
            Presentation::Active => {
                window.visible = true;
                window.style = (window.style & !OWNED) | TOPMOST;
            }
            Presentation::Passive => {
                window.visible = true;
                window.style |= OWNED;
            }
            Presentation::Hidden => window.visible = false,
        }
        if fail {
            Err("presentation failed after mutation")
        } else {
            Ok(())
        }
    }

    fn activate(&mut self, target: &u64) -> Result<(), Self::Error> {
        let mut world = self.0.lock().unwrap();
        let fail = world.step("activate");
        world.foreground = Some(*target);
        if fail {
            Err("activation confirmation failed")
        } else {
            Ok(())
        }
    }

    fn restore_focus(&mut self, focus: &u64) -> Result<(), Self::Error> {
        let mut world = self.0.lock().unwrap();
        if world.step("restore_focus") {
            return Err("saved foreground is unavailable");
        }
        world.foreground = Some(*focus);
        Ok(())
    }

    fn release_focus_if_owned(&mut self, target: &u64) -> Result<(), Self::Error> {
        let mut world = self.0.lock().unwrap();
        if world.step("release_focus") {
            return Err("focus release refused");
        }
        if world.foreground == Some(*target) {
            world.foreground = None;
        }
        Ok(())
    }

    fn observe(&mut self, _: &u64) -> Result<Observation, Self::Error> {
        let mut world = self.0.lock().unwrap();
        if world.step("observe") {
            return Err("observation unavailable");
        }
        if world.failures.front() == Some(&"mismatch") {
            world.failures.pop_front();
            return Ok(Observation {
                visible: true,
                topmost: true,
                click_through: true,
                focused: true,
            });
        }
        Ok(world.observation())
    }

    fn restore_original(&mut self, target: &u64, original: &Original) -> Result<(), Self::Error> {
        let mut world = self.0.lock().unwrap();
        let fail = world.step("restore_original");
        let window = world.windows.get_mut(target).ok_or("target closed")?;
        window.style = (window.style & !OWNED) | original.owned_style;
        window.visible = original.visible;
        if fail {
            Err("original restoration could not be confirmed")
        } else {
            Ok(())
        }
    }
}

fn setup() -> (Controller<Fake>, Arc<Mutex<World>>) {
    let world = Arc::new(Mutex::new(World {
        windows: HashMap::new(),
        foreground: Some(10),
        next_target: 100,
        calls: Vec::new(),
        failures: VecDeque::new(),
        capabilities: Capabilities::ALL,
    }));
    (Controller::new(Fake(world.clone())), world)
}

fn assert_view(controller: &Controller<Fake>, world: &Arc<Mutex<World>>, state: State) {
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

#[test]
fn absent_visibility_launches_without_activation_or_foreground_theft() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
    let world = world.lock().unwrap();
    assert_eq!(world.foreground, Some(10));
    assert_eq!(
        world.calls,
        [
            "acquire.passive",
            "snapshot",
            "present.passive",
            "release_focus",
            "observe"
        ]
    );
}

#[test]
fn active_launch_saves_the_external_foreground_before_acquisition() {
    let (mut controller, world) = setup();
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_view(&controller, &world, State::Active);
    {
        let world = world.lock().unwrap();
        assert_eq!(
            &world.calls[..3],
            ["foreground", "acquire.active", "snapshot"]
        );
    }
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_view(&controller, &world, State::Passive);
    assert_eq!(world.lock().unwrap().foreground, Some(10));
}

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
fn unsupported_requests_are_rejected_before_foreground_capture_or_mutations() {
    let (mut controller, world) = setup();
    world.lock().unwrap().capabilities.passive = false;
    let error = controller.dispatch(Event::VisibilityToggle).unwrap_err();
    assert!(matches!(error.failure.cause, Cause::Unsupported));
    assert!(world.lock().unwrap().calls.is_empty());
    world.lock().unwrap().capabilities.hidden = false;
    let error = controller.dispatch(Event::InteractionToggle).unwrap_err();
    assert!(matches!(error.failure.cause, Cause::Unsupported));
    assert!(world.lock().unwrap().calls.is_empty());
}

#[test]
fn acquisition_and_foreground_errors_do_not_create_managed_ownership() {
    for operation in ["foreground", "acquire.active"] {
        let (mut controller, world) = setup();
        world.lock().unwrap().fail(&[operation]);
        let error = controller.dispatch(Event::InteractionToggle).unwrap_err();
        assert_eq!(error.recovery, Recovery::NotNeeded);
        assert_eq!(controller.status().availability, Availability::Absent);
        assert!(world.lock().unwrap().windows.is_empty());
    }
}

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

#[test]
fn launch_and_active_capability_refusal_do_not_create_or_modify_targets() {
    for capabilities in [
        Capabilities {
            launch: false,
            ..Capabilities::ALL
        },
        Capabilities {
            active: false,
            ..Capabilities::ALL
        },
    ] {
        let (mut controller, world) = setup();
        world.lock().unwrap().capabilities = capabilities;
        assert!(matches!(
            controller
                .dispatch(Event::InteractionToggle)
                .unwrap_err()
                .failure
                .cause,
            Cause::Unsupported
        ));
        assert!(world.lock().unwrap().calls.is_empty());
        assert_eq!(controller.status().availability, Availability::Absent);
    }
}
