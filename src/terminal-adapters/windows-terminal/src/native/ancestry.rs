use super::{NativeError, ProcessIdentity, error::last};
use std::{
    collections::{HashMap, HashSet},
    os::windows::io::{FromRawHandle, OwnedHandle},
    path::Path,
};
use windows_sys::Win32::{Foundation::INVALID_HANDLE_VALUE, System::Diagnostics::ToolHelp::*};

/// Retained/live ancestry with creation-time checks. A reused upstream parent
/// invalidates this complete-chain request; verification uses a narrower prefix.
pub fn ancestry(shell_pid: u32) -> Result<Vec<ProcessIdentity>, NativeError> {
    walk(shell_pid, &parents()?, None, ProcessIdentity::open)
}
pub(super) fn to_terminal(
    shell_pid: u32,
    owner_pid: u32,
    terminal: &Path,
) -> Result<Vec<ProcessIdentity>, NativeError> {
    walk(
        shell_pid,
        &parents()?,
        Some((owner_pid, terminal)),
        ProcessIdentity::open,
    )
}
fn parents() -> Result<HashMap<u32, u32>, NativeError> {
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
    Ok(parents)
}
fn walk(
    shell_pid: u32,
    parents: &HashMap<u32, u32>,
    terminal: Option<(u32, &Path)>,
    mut open: impl FnMut(u32) -> Result<ProcessIdentity, NativeError>,
) -> Result<Vec<ProcessIdentity>, NativeError> {
    let mut chain = Vec::<ProcessIdentity>::new();
    let mut seen = HashSet::new();
    let mut pid = shell_pid;
    for _ in 0..16 {
        if pid == 0 || !seen.insert(pid) {
            break;
        }
        let identity = match open(pid) {
            Ok(value) => value,
            Err(error) if chain.is_empty() => return Err(error),
            Err(_) => break,
        };
        if let Some(child) = chain.last()
            && identity.created() > child.created()
        {
            return Err(NativeError::AncestryReused {
                child_pid: child.pid,
                parent_pid: pid,
            });
        }
        let reached = terminal.is_some_and(|(owner_pid, path)| {
            identity.pid == owner_pid
                && identity
                    .executable
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&path.to_string_lossy())
        });
        chain.push(identity);
        // Higher ancestors establish no ownership relationship to this target.
        // Do not let their stale parent PIDs invalidate the checked owner prefix.
        if reached {
            break;
        }
        pid = parents.get(&pid).copied().unwrap_or(0);
    }
    if let Some((owner_pid, path)) = terminal
        && !chain.last().is_some_and(|identity| {
            identity.pid == owner_pid
                && identity
                    .executable
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&path.to_string_lossy())
        })
    {
        return Err(NativeError::Invalid(
            "expected Terminal owner absent from verified ancestry",
        ));
    }
    Ok(chain)
}
#[cfg(test)]
#[path = "ancestry_tests.rs"]
mod tests;
