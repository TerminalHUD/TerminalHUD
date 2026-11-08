use super::super::*;

#[test]
#[ignore = "explicit interactive Windows desktop test: shows only owned helper windows and clicks within them"]
fn desktop_transitions_input_and_owned_style_restoration() {
    let sink = Fixture::launch(false, false);
    sink.focus();
    assert_ne!(
        unsafe {
            SetWindowPos(
                sink.hwnd(),
                HWND_TOPMOST,
                100,
                100,
                480,
                320,
                SWP_NOACTIVATE,
            )
        },
        0
    );
    let untouched = Fixture::launch(true, false);
    let untouched_before = (untouched.style(), unsafe {
        IsWindowVisible(untouched.hwnd())
    });
    let fixtures = Arc::new(Mutex::new(Vec::<Fixture>::new()));
    let launched = fixtures.clone();
    let backend = WindowsBackend::new(move |intent| {
        let fixture = Fixture::launch(intent == VisibleState::Passive, false);
        let receipt = fixture.claim()?;
        launched.lock().unwrap().push(fixture);
        Ok(receipt)
    })
    .unwrap();
    let mut controller = Controller::new(backend);
    controller.dispatch(Event::VisibilityToggle).unwrap();
    assert_eq!(controller.status().state, Some(State::Passive));
    assert_eq!(
        unsafe { GetForegroundWindow() },
        sink.hwnd(),
        "Passive acquisition stole focus"
    );
    let target = fixtures.lock().unwrap()[0].hwnd();
    let original_owned = 0;
    assert_eq!(
        unsafe { GetWindowLongPtrW(target, GWL_EXSTYLE) } as u32
            & (WS_EX_TRANSPARENT | WS_EX_LAYERED),
        WS_EX_TRANSPARENT | WS_EX_LAYERED
    );
    assert_eq!(
        unsafe { GetWindowLongPtrW(target, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST,
        WS_EX_TOPMOST
    );
    let point = center(target);
    wait(|| unsafe { WindowFromPoint(point) } == sink.hwnd());
    controller.dispatch(Event::InteractionToggle).unwrap();
    assert_eq!(controller.status().state, Some(State::Active));
    assert_eq!(unsafe { GetForegroundWindow() }, target);
    wait(|| unsafe { WindowFromPoint(point) } == target);
    let hidden = controller.dispatch(Event::VisibilityToggle).unwrap();
    assert!(
        hidden.focus_restore_error.is_none(),
        "foreground restoration was denied: {:?}",
        hidden.focus_restore_error
    );
    assert_eq!(unsafe { IsWindowVisible(target) }, 0);
    assert_eq!(unsafe { GetForegroundWindow() }, sink.hwnd());
    println!("PASS: native Active/Hidden and restoration to the saved owned sink");
    match controller.dispatch(Event::VisibilityToggle) {
        Ok(_) => {
            assert_eq!(controller.status().state, Some(State::Active));
            assert_eq!(unsafe { GetForegroundWindow() }, target);
            // Exit safely before nonactivating repeated transitions.
            controller.dispatch(Event::RequestPassive).unwrap();
            println!("PASS: Hidden(Active) restoration accepted by Windows foreground policy");
        }
        Err(failure) => {
            assert_eq!(failure.failure.stage, hudswitch_core::Stage::Activate);
            assert!(matches!(
                failure.failure.cause,
                hudswitch_core::Cause::Backend(Error::FocusRefused)
            ));
            assert_eq!(failure.recovery, hudswitch_core::Recovery::Passive);
            assert_eq!(controller.status().state, Some(State::Passive));
            println!(
                "OBSERVED LIMIT: Windows refused repeated activation without fresh foreground permission; actual safe Passive fallback confirmed"
            );
        }
    }
    assert_ne!(unsafe { GetForegroundWindow() }, target);
    for _ in 0..10 {
        controller.dispatch(Event::VisibilityToggle).unwrap();
        assert_eq!(unsafe { IsWindowVisible(target) }, 0);
        controller.dispatch(Event::VisibilityToggle).unwrap();
        assert_ne!(unsafe { IsWindowVisible(target) }, 0);
        assert_ne!(unsafe { GetForegroundWindow() }, target);
    }
    wait(|| unsafe { WindowFromPoint(point) } == sink.hwnd());
    let before = sink.message(WM_APP + 1);
    click_owned(point, sink.hwnd());
    wait(|| sink.message(WM_APP + 1) == before + 1);
    println!("PASS: actual guarded click routed through Passive target to owned sink");
    fixtures.lock().unwrap()[0].message(WM_APP + 2);
    controller.dispatch(Event::Quit).unwrap();
    let restored = unsafe { GetWindowLongPtrW(target, GWL_EXSTYLE) } as u32;
    assert_eq!(
        restored & (WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOPMOST),
        original_owned
    );
    assert_ne!(
        restored & WS_EX_TOOLWINDOW,
        0,
        "quit overwrote an unrelated style change"
    );
    assert_ne!(unsafe { IsWindowVisible(target) }, 0);
    assert_ne!(unsafe { IsWindow(target) }, 0, "quit closed the target");
    assert_eq!(
        (untouched.style(), unsafe {
            IsWindowVisible(untouched.hwnd())
        }),
        untouched_before
    );
    println!(
        "PASS: 20 repeated Passive/Hidden transitions; quit restored owned bits, preserved unrelated bit and left target open; separate fixture unchanged"
    );
}
