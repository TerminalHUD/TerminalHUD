use super::*;
use crate::mode::{Mode, REAL_INSTANCE};
#[test]
fn modes_are_explicit_and_cannot_route_prototype_to_real_owner() {
    let real = parse(vec!["status".into()]).unwrap();
    assert_eq!(real.mode, Mode::Real);
    assert_eq!(real.instance, REAL_INSTANCE);
    assert!(parse(vec!["run".into()]).is_err());
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
