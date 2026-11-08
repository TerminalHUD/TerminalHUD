use super::super::*;

#[test]
#[ignore = "explicit interactive Windows desktop test: owned disabled target tests activation eligibility refusal"]
fn desktop_disabled_target_refuses_activation_and_recovers_safely() {
    let sink = Fixture::launch(false, false);
    sink.focus();
    let fixture = Fixture::launch(true, false);
    unsafe {
        EnableWindow(fixture.hwnd(), 0);
    }
    assert_eq!(unsafe { IsWindowEnabled(fixture.hwnd()) }, 0);
    let mut receipt = Some(fixture.claim().unwrap());
    let backend = WindowsBackend::new(move |_| Ok(receipt.take().unwrap())).unwrap();
    let mut controller = Controller::new(backend);
    let failure = controller.dispatch(Event::InteractionToggle).unwrap_err();
    assert_eq!(failure.failure.stage, hudswitch_core::Stage::Activate);
    assert_eq!(failure.recovery, hudswitch_core::Recovery::Passive);
    assert_eq!(controller.status().state, Some(State::Passive));
    assert_eq!(unsafe { GetForegroundWindow() }, sink.hwnd());
    assert_eq!(
        fixture.style() & (WS_EX_LAYERED | WS_EX_TRANSPARENT),
        WS_EX_LAYERED | WS_EX_TRANSPARENT
    );
    controller.dispatch(Event::Quit).unwrap();
    assert_eq!(
        fixture.style() & (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST),
        0
    );
    assert_eq!(
        unsafe { IsWindowEnabled(fixture.hwnd()) },
        0,
        "quit changed an unrelated normal style"
    );
    println!(
        "PASS: backend refused ineligible disabled target activation; core recovered to actual Passive; quit restored only owned changes"
    );
}
