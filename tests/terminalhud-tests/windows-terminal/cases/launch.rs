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

#[cfg(windows)]
#[test]
fn explicit_command_rejects_extra_terminal_commands_before_process_creation() {
    use terminalhud_windows_terminal::native::{NativeError, start_with_command};
    let plan = LaunchPlan {
        identity: LaunchIdentity {
            nonce: guid(5),
            profile: guid(4),
        },
        intent: VisibleState::Active,
        primary_origin: (0, 0),
    };
    let executable = std::env::current_exe().unwrap();
    let cwd = executable.parent().unwrap();
    assert!(matches!(
        start_with_command(&plan, &executable, cwd, &[]),
        Err(NativeError::Invalid(_))
    ));
    assert!(matches!(
        start_with_command(
            &plan,
            &executable,
            cwd,
            &["fixture.exe".into(), "; new-tab ordinary-shell.exe".into()]
        ),
        Err(NativeError::Invalid(_))
    ));
}

#[cfg(windows)]
#[test]
fn normal_passive_entrypoints_refuse_before_path_checks_or_spawn() {
    use terminalhud_windows_terminal::native::{NativeError, start, start_with_command};
    let plan = LaunchPlan {
        identity: LaunchIdentity {
            nonce: guid(5),
            profile: guid(4),
        },
        intent: VisibleState::Passive,
        primary_origin: (0, 0),
    };
    let exe = std::path::Path::new("C:\\TerminalHUD-nonexistent-guard-test.exe");
    let cwd = std::path::Path::new("C:\\");
    assert!(matches!(
        start(&plan, exe, cwd),
        Err(NativeError::Unsupported(_))
    ));
    assert!(matches!(
        start_with_command(&plan, exe, cwd, &["fixture.exe".into()]),
        Err(NativeError::Unsupported(_))
    ));
}

#[cfg(windows)]
#[path = "../proof/windows/show.rs"]
mod diagnostic_show;
#[cfg(windows)]
#[test]
fn diagnostic_argument_mapping_allows_only_the_two_reviewed_show_commands() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_SHOWNA, SW_SHOWNOACTIVATE};
    assert_eq!(
        diagnostic_show::Show::parse(Some("4")).unwrap().value(),
        SW_SHOWNOACTIVATE as u16
    );
    assert_eq!(
        diagnostic_show::Show::parse(Some("8")).unwrap().value(),
        SW_SHOWNA as u16
    );
    assert!(diagnostic_show::Show::parse(None).is_err());
    for value in ["0", "1", "hide", "active", "", "4;8"] {
        assert!(diagnostic_show::Show::parse(Some(value)).is_err());
    }
}
