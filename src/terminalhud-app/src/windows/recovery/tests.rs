//! Disposable hidden window and owner process; no desktop input or activation.
use super::*;
use hudswitch_windows::{Error, LaunchedWindow, Launcher, RecoveryClaim, WindowsBackend};
use std::{
    io::{BufRead, BufReader, Read, Write},
    os::windows::io::{FromRawHandle, OwnedHandle},
    process::{Child, Command, Stdio},
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use terminalhud_core::integration::{Backend, VisibleState};
use windows_sys::Win32::{
    Foundation::*,
    System::{LibraryLoader::GetModuleHandleW, Threading::*},
    UI::WindowsAndMessaging::*,
};

const INPUT_BITS: u32 = WS_EX_LAYERED | WS_EX_TRANSPARENT;
static NEXT: AtomicUsize = AtomicUsize::new(1);

pub(super) struct HiddenWindow {
    pub(super) hwnd: HWND,
    class: Vec<u16>,
    instance: HINSTANCE,
}
impl HiddenWindow {
    pub(super) fn new() -> Self {
        let class: Vec<u16> = format!(
            "TerminalHUD.RecoveryFixture.{}.{}\0",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )
        .encode_utf16()
        .collect();
        let instance = unsafe { GetModuleHandleW(null()) };
        let descriptor = WNDCLASSW {
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            ..unsafe { std::mem::zeroed() }
        };
        assert_ne!(unsafe { RegisterClassW(&descriptor) }, 0);
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                100,
                100,
                null_mut(),
                null_mut(),
                instance,
                null(),
            )
        };
        assert!(!hwnd.is_null());
        assert_eq!(unsafe { IsWindowVisible(hwnd) }, 0);
        Self {
            hwnd,
            class,
            instance,
        }
    }
}
impl Drop for HiddenWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
            UnregisterClassW(self.class.as_ptr(), self.instance);
        }
    }
}
pub(super) struct OwnedChild(pub(super) Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
pub(super) fn open_process(pid: u32) -> OwnedHandle {
    let raw = unsafe {
        OpenProcess(
            PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        )
    };
    assert!(!raw.is_null());
    unsafe { OwnedHandle::from_raw_handle(raw) }
}
struct TestLauncher {
    receipt: Option<LaunchedWindow>,
    bridge: OwnedHandle,
    instance: String,
}
impl Launcher for TestLauncher {
    fn launch(&mut self, _: VisibleState) -> Result<LaunchedWindow, Error> {
        Ok(self.receipt.take().unwrap())
    }
    fn enroll(&mut self, claim: RecoveryClaim) -> Result<bool, Error> {
        super::enroll(&self.instance, &self.bridge, claim).map_err(Error::Launch)?;
        Ok(true)
    }
}

// Spawned explicitly by owner_death_recovers_hidden_window; never run by itself.
#[test]
#[ignore = "helper subprocess for the hidden owner-death test"]
fn owner_child() {
    let pid: u32 = std::env::var("TER29_FIXTURE_PID").unwrap().parse().unwrap();
    let hwnd = std::env::var("TER29_FIXTURE_HWND")
        .unwrap()
        .parse::<usize>()
        .unwrap() as HWND;
    let instance = std::env::var("TER29_FIXTURE_PIPE").unwrap();
    // The parent test created this exact window and retains its process identity.
    let receipt = unsafe { LaunchedWindow::from_verified_launch(hwnd, open_process(pid)) }.unwrap();
    let mut backend = WindowsBackend::new(TestLauncher {
        receipt: Some(receipt),
        bridge: open_process(pid),
        instance,
    })
    .unwrap();
    let acquired = backend.acquire(VisibleState::Active).unwrap();
    assert_eq!(acquired.target.window_handle(), hwnd);
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    unsafe {
        SetLastError(0);
    }
    let previous = unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | INPUT_BITS as isize) };
    assert!(previous != 0 || unsafe { GetLastError() } == 0);
    println!("TER29_OWNER_READY");
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read(&mut [0u8; 1]);
    // The parent terminates this exact helper before this point.
    drop(acquired);
}

#[test]
fn owner_death_recovers_hidden_window() {
    let window = HiddenWindow::new();
    let instance = format!(
        "ter29_{}_{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    start(instance.clone()).unwrap();
    let mut child = OwnedChild(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "windows::recovery::tests::owner_child",
                "--ignored",
                "--nocapture",
            ])
            .env("TER29_FIXTURE_PID", std::process::id().to_string())
            .env("TER29_FIXTURE_HWND", (window.hwnd as usize).to_string())
            .env("TER29_FIXTURE_PIPE", instance)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let output = child.0.stdout.take().unwrap();
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            if line.contains("TER29_OWNER_READY") {
                let _ = sender.send(());
                break;
            }
        }
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        pump_hidden(window.hwnd);
        if receiver.try_recv().is_ok() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "owner did not enroll and signal readiness"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        unsafe { GetWindowLongPtrW(window.hwnd, GWL_EXSTYLE) } as u32 & INPUT_BITS,
        INPUT_BITS
    );
    let marker: Vec<u16> = "HudSwitch.Acquisition.v1\0".encode_utf16().collect();
    assert!(!unsafe { GetPropW(window.hwnd, marker.as_ptr()) }.is_null());
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        pump_hidden(window.hwnd);
        let style = unsafe { GetWindowLongPtrW(window.hwnd, GWL_EXSTYLE) } as u32;
        if style & INPUT_BITS == 0 && unsafe { GetPropW(window.hwnd, marker.as_ptr()) }.is_null() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "watcher did not restore input and release claim"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_ne!(unsafe { IsWindow(window.hwnd) }, 0);
    assert_eq!(unsafe { IsWindowVisible(window.hwnd) }, 0);
    assert_eq!(
        unsafe { GetWindowLongPtrW(window.hwnd, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST,
        0
    );
}

// Dispatch only messages for the hidden owned fixture; no desktop input is sent.
pub(super) fn pump_hidden(hwnd: HWND) {
    let mut message: MSG = unsafe { std::mem::zeroed() };
    while unsafe { PeekMessageW(&mut message, hwnd, 0, 0, PM_REMOVE) } != 0 {
        unsafe {
            DispatchMessageW(&message);
        }
    }
}
