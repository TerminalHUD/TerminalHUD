use super::fixture::Primary;
use std::{
    thread,
    time::{Duration, Instant},
};
use terminalhud_app::{protocol, windows::security::Identity};
use terminalhud_core::Command;
#[test]
fn delayed_reader_receives_reply_before_server_disconnects() {
    use std::{
        fs::File,
        io::{Read, Write},
        os::windows::io::{FromRawHandle, OwnedHandle},
    };
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{
            CreateFileW, OPEN_EXISTING, SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
        },
    };
    let mut primary = Primary::start();
    let identity = Identity::current().unwrap();
    let name = identity.pipe_name(primary.instance()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let raw = loop {
        let raw = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                std::ptr::null(),
                OPEN_EXISTING,
                SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
                std::ptr::null_mut(),
            )
        };
        if raw != INVALID_HANDLE_VALUE {
            break raw;
        }
        let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        assert_eq!(error, windows_sys::Win32::Foundation::ERROR_PIPE_BUSY);
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    };
    let owned = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut file = File::from(owned);
    file.write_all(&protocol::encode_request(Command::Status))
        .unwrap();
    thread::sleep(Duration::from_millis(250));
    let mut bytes = [0; protocol::REPLY_LEN];
    file.read_exact(&mut bytes).unwrap();
    assert_eq!(
        protocol::Reply::decode(&bytes).unwrap().process_id,
        primary.pid()
    );
    file.write_all(b"THA1").unwrap();
    drop(file);
    primary.call(Command::Quit);
    primary.wait();
}
