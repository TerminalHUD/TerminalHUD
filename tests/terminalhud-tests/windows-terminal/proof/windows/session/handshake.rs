//! Bounded nonce/profile/session reply parsing with a retained fixture process.
use super::super::Result;
use std::{
    fs,
    os::windows::io::{FromRawHandle, OwnedHandle},
    path::Path,
    thread,
    time::{Duration, Instant},
};
use terminalhud_windows_terminal::{Guid, SessionProof};
use windows_sys::Win32::System::Threading::*;
pub(crate) fn read(task: &Path) -> Result<(SessionProof, OwnedHandle)> {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if let Ok(bytes) = fs::read(task.join("session-proof.json"))
            && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
        {
            let proof = SessionProof {
                nonce: Guid::parse(value["nonce"].as_str().ok_or("handshake nonce missing")?)?,
                profile: Guid::parse(
                    value["profile"]
                        .as_str()
                        .ok_or("handshake profile missing")?,
                )?,
                session: Guid::parse(
                    value["session"]
                        .as_str()
                        .ok_or("handshake session missing")?,
                )?,
                shell_pid: u32::try_from(
                    value["shellPid"].as_u64().ok_or("handshake PID missing")?,
                )?,
            };
            let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, proof.shell_pid) };
            if handle.is_null() {
                return Err("cannot retain owned fixture process".into());
            }
            let shell = unsafe { OwnedHandle::from_raw_handle(handle) };
            return Ok((proof, shell));
        }
        if Instant::now() >= deadline {
            return Err("fixture handshake timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}
