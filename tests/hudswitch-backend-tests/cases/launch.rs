use super::super::*;

#[test]
fn launch_failure_has_no_target_or_desktop_side_effects() {
    let backend =
        WindowsBackend::new(|_| Err(Error::Unsupported("test launcher unavailable"))).unwrap();
    let mut controller = Controller::new(backend);
    let failure = controller.dispatch(Event::VisibilityToggle).unwrap_err();
    assert!(matches!(
        failure.failure.cause,
        hudswitch_core::Cause::Backend(Error::Unsupported(_))
    ));
    assert_eq!(controller.status().availability, Availability::Absent);
}

#[test]
fn a_child_without_a_window_cannot_be_claimed() {
    let mut child = Command::new("cmd.exe")
        .args(["/c", "exit", "0"])
        .spawn()
        .unwrap();
    child.wait().unwrap();
    assert!(matches!(
        LaunchedWindow::from_child(&child, null_mut()),
        Err(Error::InvalidTarget)
    ));
}
