use super::{NativeError, error::last};
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::path::PathBuf;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Threading::*;
pub struct ProcessIdentity {
    pub pid: u32,
    pub(super) process: OwnedHandle,
    created: u64,
    pub executable: PathBuf,
}
impl ProcessIdentity {
    pub(super) fn created(&self) -> u64 {
        self.created
    }
    #[cfg(test)]
    pub(super) fn fixture(pid: u32, created: u64, executable: &str) -> Self {
        let mut identity = Self::open(unsafe { GetCurrentProcessId() }).unwrap();
        identity.pid = pid;
        identity.created = created;
        identity.executable = executable.into();
        identity
    }

    pub(super) fn open(pid: u32) -> Result<Self, NativeError> {
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                pid,
            )
        };
        if handle.is_null() {
            return Err(last("OpenProcess launch ancestry"));
        }
        let process = unsafe { OwnedHandle::from_raw_handle(handle) };
        if unsafe { WaitForSingleObject(handle, 0) } != WAIT_TIMEOUT {
            return Err(NativeError::Invalid("launch ancestor exited"));
        }
        let (mut created, mut exited, mut kernel, mut user) = unsafe {
            (
                std::mem::zeroed(),
                std::mem::zeroed(),
                std::mem::zeroed(),
                std::mem::zeroed(),
            )
        };
        if unsafe { GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) }
            == 0
        {
            return Err(last("GetProcessTimes launch ancestry"));
        }
        let mut path = vec![0u16; 32768];
        let mut length = path.len() as u32;
        if unsafe { QueryFullProcessImageNameW(handle, 0, path.as_mut_ptr(), &mut length) } == 0 {
            return Err(last("QueryFullProcessImageNameW"));
        }
        Ok(Self {
            pid,
            process,
            created: (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime),
            executable: PathBuf::from(String::from_utf16_lossy(&path[..length as usize])),
        })
    }
}
