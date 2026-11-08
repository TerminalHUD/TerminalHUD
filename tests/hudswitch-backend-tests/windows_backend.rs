#![cfg(windows)]

use hudswitch_core::{Availability, Backend, Controller, Event, Presentation, State, VisibleState};
use hudswitch_windows::{Error, LaunchedWindow, WindowsBackend};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::DataExchange::GlobalFindAtomW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

struct Fixture {
    child: Child,
    hwnd: isize,
}
impl Fixture {
    fn launch(passive: bool, layered: bool) -> Self {
        static DPI: std::sync::Once = std::sync::Once::new();
        DPI.call_once(|| assert_ne!(unsafe { SetProcessDPIAware() }, 0));
        let executable = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/test-window.exe");
        assert!(
            executable.is_file(),
            "first run cargo build -p hudswitch-windows --example test-window"
        );
        let mut command = Command::new(executable);
        if passive {
            command.arg("--passive");
        }
        if layered {
            command.arg("--layered");
        }
        let mut child = command.stdout(Stdio::piped()).spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stdout).read_line(&mut line);
            let _ = sender.send((result, line));
        });
        let mut fixture = Self { child, hwnd: 0 };
        let (result, line) = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("fixture did not become ready");
        result.unwrap();
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.first(), Some(&"READY"), "{line}");
        assert_eq!(fields[1].parse::<u32>().unwrap(), fixture.child.id());
        fixture.hwnd = fields[2].parse().unwrap();
        fixture
    }
    fn hwnd(&self) -> HWND {
        self.hwnd as HWND
    }
    fn claim(&self) -> Result<LaunchedWindow, Error> {
        LaunchedWindow::from_child(&self.child, self.hwnd())
    }
    fn focus(&self) {
        assert_ne!(
            unsafe { SetForegroundWindow(self.hwnd()) },
            0,
            "Windows refused foreground control. Run this explicit desktop test from a foreground native PowerShell; no foreground policy is bypassed."
        );
        wait(|| unsafe { GetForegroundWindow() } == self.hwnd());
    }
    fn message(&self, message: u32) -> usize {
        let mut result = 0;
        assert_ne!(
            unsafe {
                SendMessageTimeoutW(
                    self.hwnd(),
                    message,
                    0,
                    0,
                    SMTO_ABORTIFHUNG,
                    500,
                    &mut result,
                )
            },
            0
        );
        result
    }
    fn style(&self) -> u32 {
        (unsafe { GetWindowLongPtrW(self.hwnd(), GWL_EXSTYLE) }) as u32
    }
    fn close(&mut self) {
        if self.hwnd != 0 {
            let mut pid = 0;
            unsafe {
                GetWindowThreadProcessId(self.hwnd(), &mut pid);
            }
            if pid == self.child.id() {
                unsafe {
                    PostMessageW(self.hwnd(), WM_CLOSE, 0, 0);
                }
            }
            self.hwnd = 0;
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        while self.child.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.close();
    }
}

fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !condition() {
        assert!(Instant::now() < deadline, "desktop observation timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn layered_attributes(hwnd: HWND) -> (u32, u8, u32) {
    let (mut color, mut alpha, mut flags) = (0, 0, 0);
    assert_ne!(
        unsafe { GetLayeredWindowAttributes(hwnd, &mut color, &mut alpha, &mut flags) },
        0
    );
    (color, alpha, flags)
}

fn center(hwnd: HWND) -> POINT {
    let mut rectangle = unsafe { std::mem::zeroed() };
    assert_ne!(unsafe { GetWindowRect(hwnd, &mut rectangle) }, 0);
    POINT {
        x: (rectangle.left + rectangle.right) / 2,
        y: (rectangle.top + rectangle.bottom) / 2,
    }
}

/// Inject only after OS hit-testing confirms an owned fixture at this point.
fn click_owned(point: POINT, expected: HWND) {
    assert_eq!(
        unsafe { WindowFromPoint(point) },
        expected,
        "refusing input outside the verified owned window"
    );
    let mut old_cursor = POINT { x: 0, y: 0 };
    assert_ne!(unsafe { GetCursorPos(&mut old_cursor) }, 0);
    let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    let mouse = |flags| INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: ((point.x - left) * 65535) / (width - 1),
                dy: ((point.y - top) * 65535) / (height - 1),
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let inputs = [
        mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK),
        mouse(MOUSEEVENTF_LEFTDOWN),
        mouse(MOUSEEVENTF_LEFTUP),
    ];
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    // Cursor restoration is a movement only; no input is sent at the old point.
    unsafe {
        SetCursorPos(old_cursor.x, old_cursor.y);
    }
    assert_eq!(
        sent,
        inputs.len() as u32,
        "SendInput refused owned-window test input"
    );
}

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
