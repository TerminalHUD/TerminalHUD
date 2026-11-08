//! Private receipt regression checks. Windows are owned, hidden, and never focused.
use super::*;

#[path = "acquisition/hidden.rs"]
mod hidden;
use hidden::HiddenWindow;

#[test]
fn stale_receipt_is_rejected_before_leaving_an_acquisition_marker() {
    let window = HiddenWindow::new();
    let mut stale = Some(window.receipt());
    // Deterministically simulate a receipt whose window now belongs to a
    // different process. No real HWND-reuse schedule is assumed by this test.
    stale.as_mut().unwrap().pid ^= 1;
    let mut backend = WindowsBackend::new(move |_| Ok(stale.take().unwrap())).unwrap();
    assert!(unsafe { GetPropW(window.hwnd, backend.atom.key()) }.is_null());
    let original_style = exstyle(window.hwnd).unwrap();
    assert!(matches!(
        backend.acquire(VisibleState::Passive),
        Err(Error::InvalidTarget)
    ));
    assert!(
        unsafe { GetPropW(window.hwnd, backend.atom.key()) }.is_null(),
        "failed stale acquisition mutated the replacement's property list"
    );
    assert_eq!(exstyle(window.hwnd).unwrap(), original_style);
    assert_eq!(unsafe { IsWindowVisible(window.hwnd) }, 0);
}

#[test]
fn dropping_invalid_identity_removes_only_its_exact_owned_marker() {
    for replacement_marker in [false, true] {
        let window = HiddenWindow::new();
        let mut receipt = Some(window.receipt());
        let mut backend = WindowsBackend::new(move |_| Ok(receipt.take().unwrap())).unwrap();
        let mut acquired = backend.acquire(VisibleState::Passive).unwrap();
        assert_eq!(
            unsafe { GetPropW(window.hwnd, backend.atom.key()) } as usize,
            acquired.target.token
        );
        acquired.target.window.pid ^= 1;
        assert!(!acquired.target.valid());
        let other = acquired.target.token ^ 1;
        if replacement_marker {
            assert_ne!(
                unsafe { SetPropW(window.hwnd, backend.atom.key(), other as HANDLE) },
                0
            );
        }
        drop(acquired);
        let remaining = unsafe { GetPropW(window.hwnd, backend.atom.key()) } as usize;
        assert_eq!(remaining, if replacement_marker { other } else { 0 });
        if replacement_marker {
            // Test owns this simulated competing marker too; clean it explicitly.
            unsafe {
                RemovePropW(window.hwnd, backend.atom.key());
            }
        }
        assert_eq!(unsafe { IsWindowVisible(window.hwnd) }, 0);
    }
}
