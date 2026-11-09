use super::*;
use crate::mode::{Mode, REAL_INSTANCE};
#[test]
fn modes_are_explicit_and_cannot_route_prototype_to_real_owner() {
    let real = parse(vec!["status".into()]).unwrap();
    assert_eq!(real.mode, Mode::Real);
    assert_eq!(real.instance, REAL_INSTANCE);
    assert_eq!(parse(vec!["run".into()]).unwrap().action, Action::Run);
    assert!(
        parse(vec![
            "--prototype".into(),
            "--instance".into(),
            REAL_INSTANCE.into(),
            "quit".into()
        ])
        .is_err()
    );
    let prototype = parse(vec!["--prototype".into(), "status".into()]).unwrap();
    assert_eq!(prototype.mode, Mode::Prototype);
    assert_ne!(real.instance, prototype.instance);
}
#[test]
fn only_run_accepts_the_explicit_development_manifest() {
    let args = parse(vec![
        "--development".into(),
        "runtime.json".into(),
        "run".into(),
    ])
    .unwrap();
    assert_eq!(args.mode, Mode::Real);
    assert_eq!(args.action, Action::Run);
    assert!(args.development.is_some());
    assert!(
        parse(vec![
            "--development".into(),
            "runtime.json".into(),
            "quit".into()
        ])
        .is_err()
    );
    assert!(
        parse(vec![
            "--prototype".into(),
            "--development".into(),
            "runtime.json".into(),
            "run".into()
        ])
        .is_err()
    );
}

#[test]
fn passive_never_starts_the_gui_when_the_owner_is_absent() {
    let args = parse(vec!["passive".into()]).unwrap();
    let result = runtime_reply(
        &args,
        |_| panic!("passive must not start a host"),
        |instance, request| {
            assert_eq!(instance, REAL_INSTANCE);
            assert_eq!(
                request,
                crate::protocol::Request::Command(terminalhud_core::Command::Passive)
            );
            Err("owner unavailable".into())
        },
    );
    assert_eq!(result.unwrap_err(), "owner unavailable");
}

#[test]
fn installed_profile_setup_is_explicit() {
    assert_eq!(
        parse(vec!["profiles".into(), "plan".into()])
            .unwrap()
            .action,
        Action::ProfilePlan
    );
    assert_eq!(
        parse(vec!["profiles".into(), "apply".into()])
            .unwrap()
            .action,
        Action::ProfileApply
    );
    assert_eq!(
        parse(vec!["profiles".into(), "plan".into(), "--details".into()])
            .unwrap()
            .action,
        Action::ProfilePlanDetails
    );
    assert_eq!(
        parse(vec!["profiles".into(), "remove-plan".into()])
            .unwrap()
            .action,
        Action::ProfileRemovePlan
    );
    assert_eq!(
        parse(vec!["profiles".into(), "remove".into()])
            .unwrap()
            .action,
        Action::ProfileRemove
    );
    assert!(parse(vec!["profiles".into()]).is_err());
    assert!(parse(vec!["profiles".into(), "apply".into(), "extra".into()]).is_err());
}
