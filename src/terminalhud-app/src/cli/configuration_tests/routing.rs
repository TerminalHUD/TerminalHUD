use super::*;
use crate::cli::{Action, parse, run_with_config};

#[test]
fn local_commands_parse_and_route_without_runtime_or_opener_calls() {
    let fixture = Fixture::new();
    for (name, command) in [
        ("path", ConfigCommand::Path),
        ("validate", ConfigCommand::Validate),
        ("open", ConfigCommand::Open),
    ] {
        let parsed = parse(vec!["config".into(), name.into()]).unwrap();
        assert_eq!(parsed.action, Action::Config(command));
        assert!(parsed.development.is_none());
        let response = run_with_config(vec!["config".into(), name.into()], |routed| {
            assert_eq!(routed, command);
            Ok("local command, without startup or IPC".into())
        })
        .unwrap();
        assert_eq!(response, "local command, without startup or IPC");
    }
    let result = run_at(ConfigCommand::Path, &fixture.path, |_| {
        panic!("path must not open")
    });
    assert_eq!(result.unwrap(), fixture.path.display().to_string());
    let error = run_at(ConfigCommand::Validate, &fixture.path, |_| {
        panic!("validate must not open")
    })
    .unwrap_err();
    assert!(error.contains(&fixture.path.display().to_string()));
    assert!(!fixture.path.parent().unwrap().exists());
}

#[test]
fn runtime_flags_and_extra_arguments_are_rejected_for_local_commands() {
    for args in [
        vec!["config"],
        vec!["config", "unknown"],
        vec!["config", "path", "extra"],
        vec!["--prototype", "config", "path"],
        vec!["--prototype", "--instance", "fixture", "config", "open"],
        vec!["--development", "manifest.json", "config", "validate"],
        vec!["config", "path", "--prototype"],
        vec!["--instance", "fixture", "config", "path"],
    ] {
        assert!(
            parse(args.iter().map(|s| (*s).into()).collect()).is_err(),
            "{args:?}"
        );
    }
}
