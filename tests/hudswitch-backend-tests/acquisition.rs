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

struct EnrolledLauncher {
    receipt: Option<LaunchedWindow>,
    claim: std::rc::Rc<std::cell::RefCell<Option<RecoveryClaim>>>,
}
impl Launcher for EnrolledLauncher {
    fn launch(&mut self, _: VisibleState) -> Result<LaunchedWindow, Error> {
        Ok(self.receipt.take().unwrap())
    }
    fn enroll(&mut self, claim: RecoveryClaim) -> Result<bool, Error> {
        *self.claim.borrow_mut() = Some(claim);
        Ok(true)
    }
}

#[test]
fn watcher_restores_owned_input_and_normal_release_disarms() {
    for normal_release in [false, true] {
        let window = HiddenWindow::new();
        let claim = std::rc::Rc::new(std::cell::RefCell::new(None));
        let mut backend = WindowsBackend::new(EnrolledLauncher {
            receipt: Some(window.receipt()),
            claim: claim.clone(),
        })
        .unwrap();
        let acquired = backend.acquire(VisibleState::Active).unwrap();
        let watch = RecoveryWatch::open(claim.borrow().unwrap()).unwrap();
        // Simulate synchronous managed input changes without desktop interaction.
        set_style(&acquired.target, LAYER_BITS, LAYER_BITS).unwrap();
        if !normal_release {
            set_style(&acquired.target, WS_EX_TOOLWINDOW, WS_EX_TOOLWINDOW).unwrap();
        }
        if normal_release {
            backend
                .restore_original(&acquired.target, &acquired.original)
                .unwrap();
        }
        drop(acquired);
        if normal_release {
            assert!(matches!(watch.restore(), Err(Error::InvalidTarget)));
        } else {
            watch.restore().unwrap();
        }
        assert_eq!(exstyle(window.hwnd).unwrap() & LAYER_BITS, 0);
        if !normal_release {
            assert_ne!(exstyle(window.hwnd).unwrap() & WS_EX_TOOLWINDOW, 0);
        }
        assert_ne!(unsafe { IsWindow(window.hwnd) }, 0);
    }
}

#[test]
fn watcher_rejects_replaced_or_missing_claim_without_style_changes() {
    let window = HiddenWindow::new();
    let claim = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut backend = WindowsBackend::new(EnrolledLauncher {
        receipt: Some(window.receipt()),
        claim: claim.clone(),
    })
    .unwrap();
    let acquired = backend.acquire(VisibleState::Active).unwrap();
    let watch = RecoveryWatch::open(claim.borrow().unwrap()).unwrap();
    set_style(&acquired.target, LAYER_BITS, LAYER_BITS).unwrap();
    let before = exstyle(window.hwnd).unwrap();
    let replacement = acquired.target.token ^ 1;
    assert_ne!(
        unsafe { SetPropW(window.hwnd, backend.atom.key(), replacement as HANDLE) },
        0
    );
    assert!(matches!(watch.restore(), Err(Error::InvalidTarget)));
    assert_eq!(exstyle(window.hwnd).unwrap(), before);
    assert_eq!(
        unsafe { GetPropW(window.hwnd, backend.atom.key()) } as usize,
        replacement
    );
    unsafe {
        RemovePropW(window.hwnd, backend.atom.key());
    }
}

#[test]
fn closed_target_is_not_recovered_or_replaced() {
    let window = HiddenWindow::new();
    let claim = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut backend = WindowsBackend::new(EnrolledLauncher {
        receipt: Some(window.receipt()),
        claim: claim.clone(),
    })
    .unwrap();
    let acquired = backend.acquire(VisibleState::Active).unwrap();
    let watch = RecoveryWatch::open(claim.borrow().unwrap()).unwrap();
    drop(window);
    assert!(matches!(watch.restore(), Err(Error::InvalidTarget)));
    drop(acquired);
}

struct RefusingLauncher(Option<LaunchedWindow>);
impl Launcher for RefusingLauncher {
    fn launch(&mut self, _: VisibleState) -> Result<LaunchedWindow, Error> {
        Ok(self.0.take().unwrap())
    }
    fn enroll(&mut self, _: RecoveryClaim) -> Result<bool, Error> {
        Err(Error::Unsupported("watcher unavailable"))
    }
}
#[test]
fn refused_enrollment_clears_marker_without_applying_styles() {
    let window = HiddenWindow::new();
    let before = exstyle(window.hwnd).unwrap();
    let mut backend = WindowsBackend::new(RefusingLauncher(Some(window.receipt()))).unwrap();
    assert!(backend.acquire(VisibleState::Active).is_err());
    assert!(unsafe { GetPropW(window.hwnd, backend.atom.key()) }.is_null());
    assert_eq!(exstyle(window.hwnd).unwrap(), before);
}

#[test]
fn partial_recovery_failure_is_reported_and_keeps_claim() {
    let window = HiddenWindow::new();
    let claim = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut backend = WindowsBackend::new(EnrolledLauncher {
        receipt: Some(window.receipt()),
        claim: claim.clone(),
    })
    .unwrap();
    let acquired = backend.acquire(VisibleState::Active).unwrap();
    let watch = RecoveryWatch::open(claim.borrow().unwrap()).unwrap();
    set_style(&acquired.target, LAYER_BITS, LAYER_BITS).unwrap();
    recovery::fail_after_input_once();
    assert!(matches!(watch.restore(), Err(Error::Unsupported(_))));
    assert_eq!(exstyle(window.hwnd).unwrap() & LAYER_BITS, 0);
    assert_eq!(
        unsafe { GetPropW(window.hwnd, backend.atom.key()) } as usize,
        acquired.target.token
    );
    // The retained claim permits an explicit retry after a reported failure.
    watch.restore().unwrap();
}

#[test]
fn zero_token_cannot_claim_an_unmarked_window() {
    let window = HiddenWindow::new();
    let mut pid = 0;
    let tid = unsafe { GetWindowThreadProcessId(window.hwnd, &mut pid) };
    let before = exstyle(window.hwnd).unwrap();
    let claim = RecoveryClaim {
        hwnd: window.hwnd as usize,
        pid,
        tid,
        token: 0,
        exstyle: before,
        visible: false,
    };
    assert!(matches!(
        RecoveryWatch::open(claim),
        Err(Error::InvalidTarget)
    ));
    assert_eq!(exstyle(window.hwnd).unwrap(), before);
}

#[test]
fn release_confirmation_requires_original_input_bits_and_no_marker() {
    let window = HiddenWindow::new();
    let claim = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut backend = WindowsBackend::new(EnrolledLauncher {
        receipt: Some(window.receipt()),
        claim: claim.clone(),
    })
    .unwrap();
    let acquired = backend.acquire(VisibleState::Active).unwrap();
    let watch = RecoveryWatch::open(claim.borrow().unwrap()).unwrap();
    assert!(!watch.confirm_released().unwrap());
    set_style(&acquired.target, LAYER_BITS, LAYER_BITS).unwrap();
    unsafe {
        RemovePropW(window.hwnd, backend.atom.key());
    }
    assert!(!watch.confirm_released().unwrap());
    // Restore the marker only in this owned fixture so normal cleanup can run.
    assert_ne!(
        unsafe {
            SetPropW(
                window.hwnd,
                backend.atom.key(),
                acquired.target.token as HANDLE,
            )
        },
        0
    );
    backend
        .restore_original(&acquired.target, &acquired.original)
        .unwrap();
    assert!(watch.confirm_released().unwrap());
}

#[test]
fn release_confirmation_rejects_foreign_or_uncertain_identity_and_accepts_gone_target() {
    let window = HiddenWindow::new();
    let parent = HiddenWindow::new();
    let claim = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut backend = WindowsBackend::new(EnrolledLauncher {
        receipt: Some(window.receipt()),
        claim: claim.clone(),
    })
    .unwrap();
    let acquired = backend.acquire(VisibleState::Active).unwrap();
    let watch = RecoveryWatch::open(claim.borrow().unwrap()).unwrap();
    let foreign = acquired.target.token ^ 1;
    assert_ne!(
        unsafe { SetPropW(window.hwnd, backend.atom.key(), foreign as HANDLE) },
        0
    );
    assert!(!watch.confirm_released().unwrap());
    assert_ne!(
        unsafe {
            SetPropW(
                window.hwnd,
                backend.atom.key(),
                acquired.target.token as HANDLE,
            )
        },
        0
    );
    unsafe {
        SetParent(window.hwnd, parent.hwnd);
    }
    assert!(watch.confirm_released().is_err());
    unsafe {
        SetParent(window.hwnd, null_mut());
    }
    drop(window);
    assert!(watch.confirm_released().unwrap());
}
