//! No visible UI: isolated prototype owner, normal IPC Quit and bounded fallback.
use super::*;
use crate::windows::{ipc, native::Handle};
use std::{
    fs::OpenOptions,
    ptr, thread,
    time::{Duration, Instant},
};
use terminalhud_core::Command;
use windows_sys::Win32::{
    Foundation::*, Security::SECURITY_ATTRIBUTES, Storage::FileSystem::ReadFile,
    System::Pipes::CreatePipe,
};
#[test]
fn gui_handle_allowlist_does_not_retain_an_unrelated_inheritable_capture_writer() {
    let _serial = super::INHERITANCE_TEST.lock().unwrap();
    let mut read = ptr::null_mut();
    let mut write = ptr::null_mut();
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    assert_ne!(
        unsafe { CreatePipe(&mut read, &mut write, &attributes, 0) },
        0
    );
    let read = unsafe { Handle::from_raw(read, "capture reader").unwrap() };
    let write = unsafe { Handle::from_raw(write, "capture writer").unwrap() };
    let instance = format!("spawn-{}", std::process::id());
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("terminalhud-shell.exe");
    let diagnostic = crate::windows::development::temporary_directory()
        .unwrap()
        .join(format!(
            "TerminalHUD-spawn-test-{}.log",
            terminalhud_windows_terminal::native::fresh_guid()
                .unwrap()
                .bare()
        ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&diagnostic)
        .unwrap();
    let args = [
        "--prototype",
        "--headless",
        "--instance",
        &instance,
        "--exit-after-ms",
        "5000",
    ]
    .map(Into::into);
    let child = spawn(&binary, &args, &file).unwrap();
    drop(write);
    let started = Instant::now();
    let reader = thread::spawn(move || {
        let mut byte = 0u8;
        let mut count = 0;
        let ok = unsafe { ReadFile(read.raw(), &mut byte, 1, &mut count, ptr::null_mut()) };
        (ok, count, unsafe { GetLastError() })
    });
    let (ok, count, error) = reader.join().unwrap();
    let elapsed = started.elapsed();
    assert!(ok != 0 && count == 0 || error == ERROR_BROKEN_PIPE);
    // If inheritance regresses, the 5s prototype fallback still bounds the read.
    assert!(
        elapsed < Duration::from_secs(2),
        "capture EOF waited for a long-lived GUI: {elapsed:?}"
    );
    assert_eq!(
        child.try_wait().unwrap(),
        None,
        "GUI must still be alive when EOF is received"
    );
    let reply = ipc::call_for_process(&instance, child.pid, Command::Quit).unwrap();
    assert_eq!(reply.code, crate::protocol::ReplyCode::Ok);
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    drop(file);
    std::fs::remove_file(diagnostic).unwrap();
}
