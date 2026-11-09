//! Hidden-window tests of session completion ordering and owner death.
use super::tests::{HiddenWindow, OwnedChild, open_process, pump_hidden};
use crate::windows::native::Handle;
use hudswitch_windows::{Error, LaunchedWindow, Launcher, RecoveryClaim, WindowsBackend};
use std::{
    cell::RefCell,
    io::{BufRead, BufReader, Read, Write},
    os::windows::io::OwnedHandle,
    process::{Command, Stdio},
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use terminalhud_core::integration::{Backend, VisibleState};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};
const INPUT_BITS: u32 = WS_EX_LAYERED | WS_EX_TRANSPARENT;
static NEXT: AtomicUsize = AtomicUsize::new(1);
fn identities() -> (String, String) {
    let nonce = format!(
        "{{{:08x}-0000-4000-8000-{:012x}}}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let instance = nonce.replace(['{', '}', '-'], "");
    (
        instance,
        format!(
            "{nonce} {{11111111-1111-4111-8111-111111111111}} {{22222222-2222-4222-8222-222222222222}}"
        ),
    )
}
struct LauncherFixture {
    receipt: Option<LaunchedWindow>,
    bridge: OwnedHandle,
    instance: String,
    pipe: Rc<RefCell<Option<Handle>>>,
}
impl Launcher for LauncherFixture {
    fn launch(&mut self, _: VisibleState) -> Result<LaunchedWindow, Error> {
        Ok(self.receipt.take().unwrap())
    }
    fn enroll(&mut self, claim: RecoveryClaim) -> Result<bool, Error> {
        *self.pipe.borrow_mut() = Some(
            super::enroll_session(&self.instance, &self.bridge, claim).map_err(Error::Launch)?,
        );
        Ok(true)
    }
}
#[test]
#[ignore = "spawned only by hidden recovery completion fixtures"]
fn owner_fixture() {
    let pid: u32 = std::env::var("TER29_FIXTURE_PID").unwrap().parse().unwrap();
    let hwnd = std::env::var("TER29_FIXTURE_HWND")
        .unwrap()
        .parse::<usize>()
        .unwrap() as HWND;
    let instance = std::env::var("TER29_FIXTURE_PIPE").unwrap();
    let key = std::env::var("TER29_FIXTURE_KEY").unwrap();
    let mode = std::env::var("TER29_FIXTURE_MODE").unwrap();
    let bridge = open_process(pid);
    let pipe = Rc::new(RefCell::new(None));
    let receipt = unsafe { LaunchedWindow::from_verified_launch(hwnd, open_process(pid)) }.unwrap();
    let mut backend = WindowsBackend::new(LauncherFixture {
        receipt: Some(receipt),
        bridge: open_process(pid),
        instance,
        pipe: pipe.clone(),
    })
    .unwrap();
    let acquired = backend.acquire(VisibleState::Active).unwrap();
    let original = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    if mode != "inflight" {
        unsafe {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, original | INPUT_BITS as isize);
        }
    }
    println!("FIXTURE_READY");
    std::io::stdout().flush().unwrap();
    if mode == "inflight" {
        let _ = std::io::stdin().read(&mut [0u8; 1]);
        println!("FIXTURE_STYLE_START");
        std::io::stdout().flush().unwrap();
        unsafe {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, original | INPUT_BITS as isize);
        }
        println!("FIXTURE_STYLE_DONE");
        std::io::stdout().flush().unwrap();
    }
    let test_key = if mode == "stale" {
        format!("{key} stale")
    } else {
        key.clone()
    };
    let result = super::completion(pipe.borrow().as_ref().unwrap(), &bridge, &test_key);
    if mode == "stale" {
        assert!(result.is_err());
    } else {
        result.unwrap();
    }
    println!("FIXTURE_SEEN");
    std::io::stdout().flush().unwrap();
    if mode == "premature" || mode == "late_ack" {
        let release_key = if mode == "late_ack" {
            format!("{key} stale")
        } else {
            key.clone()
        };
        assert!(super::release(pipe.borrow().as_ref().unwrap(), &release_key).is_err());
        println!("FIXTURE_DENIED");
        std::io::stdout().flush().unwrap();
    }
    let _ = std::io::stdin().read(&mut [0u8; 1]);
    if mode != "release" && mode != "inflight" {
        return;
    }
    // Simulate a queued passive style write on this same GUI thread immediately
    // before normal restoration. The fixture stays hidden and never activates.
    if mode == "release" {
        unsafe {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, original | INPUT_BITS as isize);
        }
    }
    unsafe {
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, original);
    }
    backend
        .restore_original(&acquired.target, &acquired.original)
        .unwrap();
    super::release(pipe.borrow().as_ref().unwrap(), &key).unwrap();
    println!("FIXTURE_RELEASED");
    std::io::stdout().flush().unwrap();
}
fn wait_message(window: &HiddenWindow, receiver: &mpsc::Receiver<String>, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        pump_hidden(window.hwnd);
        if let Ok(message) = receiver.try_recv()
            && message == expected
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "fixture did not signal {expected}"
        );
        thread::sleep(Duration::from_millis(5));
    }
}
fn run(mode: &str) {
    let window = HiddenWindow::new();
    let (instance, key) = identities();
    let server = super::start_session(instance.clone(), key.clone()).unwrap();
    let mut child = OwnedChild(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "windows::recovery::session_tests::owner_fixture",
                "--ignored",
                "--nocapture",
            ])
            .env("TER29_FIXTURE_PID", std::process::id().to_string())
            .env("TER29_FIXTURE_HWND", (window.hwnd as usize).to_string())
            .env("TER29_FIXTURE_PIPE", &instance)
            .env("TER29_FIXTURE_KEY", &key)
            .env("TER29_FIXTURE_MODE", mode)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let output = child.0.stdout.take().unwrap();
    let (messages_tx, messages_rx) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            for marker in [
                "FIXTURE_READY",
                "FIXTURE_STYLE_START",
                "FIXTURE_STYLE_DONE",
                "FIXTURE_SEEN",
                "FIXTURE_DENIED",
                "FIXTURE_RELEASED",
            ] {
                if line.contains(marker) {
                    let _ = messages_tx.send(marker.to_owned());
                }
            }
        }
    });
    wait_message(&window, &messages_rx, "FIXTURE_READY");
    if mode == "inflight" {
        child.0.stdin.as_mut().unwrap().write_all(b"x").unwrap();
        assert_eq!(
            messages_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            "FIXTURE_STYLE_START"
        );
        // Do not pump the owned HWND yet: the cross-process style call remains in flight.
        thread::sleep(Duration::from_millis(50));
        assert!(
            messages_rx.try_recv().is_err(),
            "style change was not blocked in flight"
        );
    } else {
        assert_eq!(
            unsafe { GetWindowLongPtrW(window.hwnd, GWL_EXSTYLE) } as u32 & INPUT_BITS,
            INPUT_BITS
        );
    }
    let sent = server.completion_written();
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = done_tx.send(server.finish());
    });
    if mode == "inflight" {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !sent.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "completion frame was not sent");
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            messages_rx.try_recv().is_err(),
            "in-flight style completed before delivery"
        );
        assert!(
            done_rx.try_recv().is_err(),
            "bridge returned during in-flight style write"
        );
    }
    wait_message(&window, &messages_rx, "FIXTURE_SEEN");
    assert!(
        done_rx.try_recv().is_err(),
        "bridge returned before UI release or GUI death"
    );
    if mode == "premature" || mode == "late_ack" {
        wait_message(&window, &messages_rx, "FIXTURE_DENIED");
        assert!(done_rx.try_recv().is_err(), "unrestored claim was accepted");
    }
    if mode == "release" || mode == "inflight" {
        child.0.stdin.as_mut().unwrap().write_all(b"x").unwrap();
        wait_message(&window, &messages_rx, "FIXTURE_RELEASED");
    } else {
        child.0.kill().unwrap();
        child.0.wait().unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        pump_hidden(window.hwnd);
        match done_rx.try_recv() {
            Ok(result) => {
                result.unwrap();
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(error) => panic!("watcher ended unexpectedly: {error}"),
        }
        assert!(Instant::now() < deadline, "watcher did not complete");
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        unsafe { GetWindowLongPtrW(window.hwnd, GWL_EXSTYLE) } as u32 & INPUT_BITS,
        0
    );
    assert_ne!(unsafe { IsWindow(window.hwnd) }, 0);
    assert_eq!(unsafe { IsWindowVisible(window.hwnd) }, 0);
    assert_eq!(
        unsafe { GetWindowLongPtrW(window.hwnd, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST,
        0
    );
}
#[test]
fn child_completion_waits_for_serialized_release() {
    run("release");
}
#[test]
fn in_flight_passive_style_finishes_before_serialized_release() {
    run("inflight");
}
#[test]
fn late_ack_for_another_session_never_releases_the_claim() {
    run("late_ack");
}
#[test]
fn premature_release_ack_is_rejected_until_owner_death() {
    run("premature");
}
#[test]
fn gui_death_after_child_completion_recovers_without_ack() {
    run("death");
}
#[test]
fn stale_completion_never_releases_a_current_claim() {
    run("stale");
}
#[test]
fn child_completion_before_enrollment_cancels_accept() {
    let (instance, key) = identities();
    let server = super::start_session(instance.clone(), key).unwrap();
    server.finish().unwrap();
    assert!(
        crate::windows::ipc::connect(
            &crate::windows::security::Identity::current().unwrap(),
            &instance,
            Duration::from_millis(50)
        )
        .is_err()
    );
}
