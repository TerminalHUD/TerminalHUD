use super::*;
const SINK: Window = Window {
    hwnd: 10,
    pid: 20,
    tid: 30,
};
const HUD: Window = Window {
    hwnd: 11,
    pid: 21,
    tid: 31,
};
fn samples() -> Vec<Sample> {
    vec![
        Sample {
            elapsed_us: 0,
            stage: Stage::Startup,
            input_tick: Some(90),
            foreground: Some(SINK),
        },
        Sample {
            elapsed_us: 10_000,
            stage: Stage::Startup,
            input_tick: Some(90),
            foreground: Some(HUD),
        },
        Sample {
            elapsed_us: 20_000,
            stage: Stage::InspectActive,
            input_tick: Some(90),
            foreground: Some(HUD),
        },
        Sample {
            elapsed_us: 30_000,
            stage: Stage::Quit,
            input_tick: Some(90),
            foreground: Some(SINK),
        },
    ]
}
fn assess(samples: &[Sample]) -> Assessment {
    classify(
        samples,
        SINK,
        Some(HUD),
        Span {
            start_us: 0,
            end_us: 40_000,
            through_quit: true,
        },
        false,
    )
}
#[test]
fn intended_owned_focus_sequence_reports_only_no_disturbance_observed() {
    assert_eq!(assess(&samples()).label(), "NO_DISTURBANCE_OBSERVED");
}
#[test]
fn activity_inside_the_sink_is_inconclusive_without_foreground_change() {
    let mut s = samples();
    s[0].input_tick = Some(89);
    assert!(assess(&s).observed_input_activity);
    assert_eq!(assess(&s).label(), "INCONCLUSIVE");
}
#[test]
fn nonmonotonic_input_ticks_are_changes_without_claiming_source_or_direction() {
    let mut s = samples();
    s[1].input_tick = Some(2);
    assert!(assess(&s).observed_input_activity);
    s[1].input_tick = Some(u32::MAX);
    assert!(assess(&s).observed_input_activity);
}
#[test]
fn a_third_foreground_or_reused_window_identity_is_inconclusive() {
    let mut s = samples();
    s[1].foreground = Some(Window { pid: 99, ..HUD });
    assert!(assess(&s).unexpected_foreground);
}
#[test]
fn an_owned_sink_during_active_inspection_is_not_an_allowed_restore() {
    let mut s = samples();
    s[2].foreground = Some(SINK);
    assert!(assess(&s).unexpected_foreground);
    s[2].stage = Stage::InspectRecovered;
    assert_eq!(assess(&s).label(), "NO_DISTURBANCE_OBSERVED");
}
#[test]
fn missing_null_unstable_failed_or_late_observations_are_inconclusive() {
    let mut s = samples();
    s[1].foreground = None;
    assert!(assess(&s).insufficient_observation);
    let mut s = samples();
    s[1].input_tick = None;
    assert!(assess(&s).insufficient_observation);
    let mut s = samples();
    s.pop();
    assert!(assess(&s).insufficient_observation);
    assert!(
        classify(
            &samples(),
            SINK,
            Some(HUD),
            Span {
                start_us: 0,
                end_us: 200_001,
                through_quit: true
            },
            false
        )
        .insufficient_observation
    );
    assert!(
        classify(
            &samples(),
            SINK,
            Some(HUD),
            Span {
                start_us: 0,
                end_us: 40_000,
                through_quit: true
            },
            true
        )
        .insufficient_observation
    );
    assert!(
        classify(
            &samples(),
            SINK,
            None,
            Span {
                start_us: 0,
                end_us: 40_000,
                through_quit: true
            },
            false
        )
        .insufficient_observation
    );
}
#[test]
fn setup_is_separate_from_protected_startup() {
    let mut s = samples();
    s.insert(
        0,
        Sample {
            stage: Stage::Setup,
            input_tick: Some(1),
            foreground: None,
            ..s[0]
        },
    );
    assert_eq!(assess(&s).label(), "NO_DISTURBANCE_OBSERVED");
}

#[test]
fn cleanup_activity_does_not_retroactively_explain_a_startup_result() {
    let mut s = samples();
    s[3].input_tick = Some(91);
    let startup = classify(
        &s,
        SINK,
        Some(HUD),
        Span {
            start_us: 0,
            end_us: 20_000,
            through_quit: false,
        },
        false,
    );
    assert_eq!(startup.label(), "NO_DISTURBANCE_OBSERVED");
    assert!(assess(&s).observed_input_activity);
}
#[test]
fn reversed_phase_order_is_insufficient_even_with_owned_foreground() {
    let mut s = samples();
    s[1].stage = Stage::Quit;
    assert!(assess(&s).insufficient_observation);
}
