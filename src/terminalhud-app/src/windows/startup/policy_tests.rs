use super::*;
#[test]
fn only_exact_receipted_reg_sz_is_owned() {
    let command = r#""C:\Users\A B\TerminalHUD\terminalhud-shell.exe""#;
    let owned = vec![command.to_string()];
    assert_eq!(
        prepare(command, &owned, None, true).unwrap(),
        Some(RegistryValue::command(command))
    );
    let mut foreign = RegistryValue::command(command);
    foreign.kind = 2;
    assert!(prepare(command, &owned, Some(&foreign), false).is_err());
    foreign = RegistryValue::command(&(command.to_string() + " --prototype"));
    assert!(prepare(command, &owned, Some(&foreign), true).is_err());
    assert_eq!(
        prepare(
            command,
            &owned,
            Some(&RegistryValue::command(command)),
            false
        )
        .unwrap(),
        None
    );
}
#[test]
fn fake_registry_commit_detects_conflicts_and_partial_failures() {
    use std::cell::RefCell;
    let old = Some(RegistryValue::command("old"));
    let new = Some(RegistryValue::command("new"));
    let registry = RefCell::new(old.clone());
    apply(
        &old,
        &new,
        || Ok(registry.borrow().clone()),
        |v| {
            *registry.borrow_mut() = v.clone();
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(*registry.borrow(), new);
    assert!(
        apply(
            &old,
            &None,
            || Ok(registry.borrow().clone()),
            |_| panic!("must preserve conflict")
        )
        .is_err()
    );
    let error = apply(
        &new,
        &None,
        || Ok(registry.borrow().clone()),
        |v| {
            *registry.borrow_mut() = v.clone();
            Err("injected failure".into())
        },
    )
    .unwrap_err();
    assert!(error.contains("re-read before retrying"));
    assert_eq!(*registry.borrow(), None);
}
#[test]
fn upgrade_owns_previous_exact_command() {
    let old = RegistryValue::command("\"old.exe\"");
    assert_eq!(
        prepare("\"new.exe\"", &["\"old.exe\"".into()], Some(&old), true).unwrap(),
        Some(RegistryValue::command("\"new.exe\""))
    );
    assert!(prepare("new", &[], Some(&old), false).is_err());
}
