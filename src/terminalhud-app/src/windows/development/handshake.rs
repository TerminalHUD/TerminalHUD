use super::error;
use std::{
    fs, io,
    os::windows::io::{FromRawHandle, OwnedHandle},
    path::Path,
};
use terminalhud_windows_terminal::{Guid, LaunchIdentity, SessionProof};
use windows_sys::Win32::System::Threading::*;
pub(in crate::windows) fn read(
    path: &Path,
    identity: &LaunchIdentity,
) -> io::Result<(SessionProof, OwnedHandle)> {
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(path.join("session.json"))?).map_err(error)?;
    let guid = |key| {
        Guid::parse(
            value[key]
                .as_str()
                .ok_or_else(|| error("session identity field missing"))?,
        )
        .map_err(error)
    };
    let proof = SessionProof {
        nonce: guid("nonce")?,
        profile: guid("profile")?,
        session: guid("session")?,
        shell_pid: u32::try_from(
            value["shellPid"]
                .as_u64()
                .ok_or_else(|| error("session PID missing"))?,
        )
        .map_err(error)?,
    };
    if proof.nonce != identity.nonce || proof.profile != identity.profile {
        return Err(error("session identity mismatch"));
    }
    let handle = unsafe {
        OpenProcess(
            PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            proof.shell_pid,
        )
    };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    Ok((proof, unsafe { OwnedHandle::from_raw_handle(handle) }))
}
