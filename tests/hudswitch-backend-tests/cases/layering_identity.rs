use super::super::*;

#[test]
#[ignore = "explicit interactive Windows desktop test: owned fixtures only"]
fn desktop_layered_preservation_identity_and_activation_failure() {
    let sink = Fixture::launch(false, false);
    sink.focus();
    let mut fixture = Fixture::launch(true, true);
    let original = layered_attributes(fixture.hwnd());
    let receipt = fixture.claim().unwrap();
    let mut backend =
        WindowsBackend::new(move |_| Err(Error::Unsupported("unused launcher"))).unwrap();
    // Use a one-shot launcher so the backend performs its full snapshot/claim path.
    let mut receipt = Some(receipt);
    let mut backend2 = WindowsBackend::new(move |_| Ok(receipt.take().unwrap())).unwrap();
    let acquired = backend2.acquire(VisibleState::Passive).unwrap();
    let existing_style = fixture.style();
    let mut duplicate = Some(fixture.claim().unwrap());
    let mut other_owner = WindowsBackend::new(move |_| Ok(duplicate.take().unwrap())).unwrap();
    assert!(matches!(
        other_owner.acquire(VisibleState::Passive),
        Err(Error::AlreadyManaged)
    ));
    assert_eq!(fixture.style(), existing_style);
    backend2
        .present(&acquired.target, Presentation::Passive)
        .unwrap();
    assert_eq!(layered_attributes(fixture.hwnd()), original);
    backend2
        .present(&acquired.target, Presentation::Active)
        .unwrap();
    assert_eq!(layered_attributes(fixture.hwnd()), original);
    backend2
        .restore_original(&acquired.target, &acquired.original)
        .unwrap();
    assert_eq!(layered_attributes(fixture.hwnd()), original);
    assert_eq!(fixture.style() & WS_EX_LAYERED, WS_EX_LAYERED);
    assert!(
        !backend.is_valid(&acquired.target).unwrap(),
        "detached marker still authorized mutations"
    );

    let receipt = fixture.claim().unwrap();
    let mut receipt = Some(receipt);
    let mut backend = WindowsBackend::new(move |_| Ok(receipt.take().unwrap())).unwrap();
    let acquired = backend.acquire(VisibleState::Passive).unwrap();
    let name: Vec<_> = "HudSwitch.Acquisition.v1\0"
        .to_owned()
        .encode_utf16()
        .collect();
    let atom = unsafe { GlobalFindAtomW(name.as_ptr()) };
    assert_ne!(atom, 0);
    unsafe {
        RemovePropW(fixture.hwnd(), atom as usize as *const u16);
    }
    assert_ne!(unsafe { IsWindow(fixture.hwnd()) }, 0);
    assert!(
        !backend.is_valid(&acquired.target).unwrap(),
        "live HWND without its acquisition marker was accepted"
    );
    let snapshot = (fixture.style(), unsafe { IsWindowVisible(fixture.hwnd()) });
    assert!(matches!(
        backend.present(&acquired.target, Presentation::Hidden),
        Err(Error::InvalidTarget)
    ));
    assert_eq!(
        (fixture.style(), unsafe { IsWindowVisible(fixture.hwnd()) }),
        snapshot
    );
    fixture.close();
    assert!(!backend.is_valid(&acquired.target).unwrap());
    println!(
        "PASS: existing layered alpha/color key unchanged; detached/live-unmarked/closed target rejected"
    );
}
