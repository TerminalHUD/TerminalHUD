use super::{NativeError, error::last};
use std::collections::{HashMap, HashSet};
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::path::PathBuf;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::{Diagnostics::ToolHelp::*, Threading::*};
pub struct ProcessIdentity {
    pub pid: u32,
    pub(super) process: OwnedHandle,
    created: u64,
    pub executable: PathBuf,
}
impl ProcessIdentity {
    fn open(pid: u32) -> Result<Self, NativeError> {
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

/// Live process ancestry with monotonic creation-time checks; retain the handles
/// through window verification so a reused PID is not treated as proof.
pub fn ancestry(shell_pid: u32) -> Result<Vec<ProcessIdentity>, NativeError> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(last("CreateToolhelp32Snapshot"));
    }
    let _snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot) };
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let mut parents = HashMap::new();
    let mut found = unsafe { Process32FirstW(snapshot, &mut entry) };
    while found != 0 {
        parents.insert(entry.th32ProcessID, entry.th32ParentProcessID);
        found = unsafe { Process32NextW(snapshot, &mut entry) };
    }
    let mut chain = Vec::<ProcessIdentity>::new();
    let mut seen = HashSet::new();
    let mut pid = shell_pid;
    for _ in 0..16 {
        if pid == 0 || !seen.insert(pid) {
            break;
        }
        let identity = match ProcessIdentity::open(pid) {
            Ok(value) => value,
            Err(error) if chain.is_empty() => return Err(error),
            Err(_) => break,
        };
        if chain
            .last()
            .is_some_and(|child| identity.created > child.created)
        {
            return Err(NativeError::Invalid("launch parent PID was reused"));
        }
        chain.push(identity);
        pid = parents.get(&pid).copied().unwrap_or(0);
    }
    Ok(chain)
}
