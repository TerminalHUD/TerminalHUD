use super::*;
use hudswitch_core::VisibleState;
#[test]
fn launch_arguments_are_isolated_fullscreen_and_do_not_use_mru_window() {
    let plan = LaunchPlan {
        identity: LaunchIdentity {
            nonce: guid(5),
            profile: guid(4),
        },
        intent: VisibleState::Passive,
        primary_origin: (0, 0),
    };
    let arguments = plan.arguments();
    assert_eq!(
        &arguments[..2],
        ["--window", plan.identity.window_name().as_str()]
    );
    assert!(arguments.contains(&"--fullscreen".into()));
    assert!(arguments.contains(&guid(4).to_string()));
    assert!(arguments.contains(&"--suppressApplicationTitle".into()));
    assert!(!arguments.contains(&"last".into()));
}
