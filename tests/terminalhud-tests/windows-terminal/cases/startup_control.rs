#[path = "../proof/windows/startup/presentation.rs"]
mod presentation;
use super::*;
use presentation::Presentation;

#[test]
fn control_parser_refuses_unknown_extra_or_wrong_show_mode() {
    assert_eq!(Presentation::parse(8, &[]), Ok(Presentation::Fullscreen));
    assert_eq!(Presentation::parse(4, &[]), Ok(Presentation::Fullscreen));
    assert_eq!(
        Presentation::parse(8, &["--windowed-control".into()]),
        Ok(Presentation::WindowedControl)
    );
    for (show, flags) in [
        (4, vec!["--windowed-control".into()]),
        (8, vec!["--windowed".into()]),
        (8, vec!["--windowed-control".into(), "extra".into()]),
    ] {
        assert!(Presentation::parse(show, &flags).is_err());
    }
}
#[test]
fn windowed_control_removes_only_fullscreen_before_unchanged_fixture_arguments() {
    let plan = LaunchPlan {
        identity: LaunchIdentity {
            nonce: guid(5),
            profile: guid(4),
        },
        intent: hudswitch_core::VisibleState::Passive,
        primary_origin: (0, 0),
    };
    let fixture = [
        "C:\\task\\fixture\\terminal-proof.exe",
        "--shell",
        "C:\\task",
        "nonce",
    ];
    let mut full = Presentation::Fullscreen.arguments(&plan).unwrap();
    assert_eq!(full, plan.arguments());
    full.extend(fixture.map(String::from));
    let mut control = Presentation::WindowedControl.arguments(&plan).unwrap();
    control.extend(fixture.map(String::from));
    let expected = full
        .into_iter()
        .filter(|argument| argument != "--fullscreen")
        .collect::<Vec<_>>();
    assert_eq!(control, expected);
    assert_eq!(&control[control.len() - 4..], fixture);
    assert_eq!(
        &control[..2],
        ["--window", plan.identity.window_name().as_str()]
    );
}
