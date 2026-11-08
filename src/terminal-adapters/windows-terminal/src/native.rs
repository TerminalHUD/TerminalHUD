//! Native launch and process/window correlation; no profile installation or style changes.
use crate::{Guid, LaunchIdentity, LaunchPlan, SessionProof, WindowCandidate, correlate};
use hudswitch_core::VisibleState;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::path::{Path, PathBuf};
use std::ptr::null;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Com::CoCreateGuid;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug)]
pub enum NativeError {
    Win32(&'static str, u32),
    Invalid(&'static str),
    Correlation(crate::Error),
}
impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NativeError {}
fn last(operation: &'static str) -> NativeError {
    NativeError::Win32(operation, unsafe { GetLastError() })
}
fn wide(value: &std::ffi::OsStr) -> Result<Vec<u16>, NativeError> {
    let mut value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err(NativeError::Invalid("NUL in native path/argument"));
    }
    value.push(0);
    Ok(value)
}

pub fn fresh_guid() -> Result<Guid, NativeError> {
    let mut value = windows_sys::core::GUID::default();
    let result = unsafe { CoCreateGuid(&mut value) };
    if result < 0 {
        return Err(NativeError::Win32("CoCreateGuid", result as u32));
    }
    let tail: String = value.data4[2..]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Guid::parse(&format!(
        "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{tail}}}",
        value.data1, value.data2, value.data3, value.data4[0], value.data4[1]
    ))
    .map_err(NativeError::Correlation)
}

/// Standard Windows argv quoting; this does not invoke a command shell.
fn quote(value: &str) -> String {
    let mut output = String::from("\"");
    let mut slashes = 0;
    for character in value.chars() {
        if character == '\\' {
            slashes += 1;
            continue;
        }
        if character == '"' {
            output.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
        } else {
            output.extend(std::iter::repeat_n('\\', slashes));
        }
        output.push(character);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n('\\', slashes * 2));
    output.push('"');
    output
}

pub struct LauncherProcess {
    pub pid: u32,
    _process: OwnedHandle,
}

/// Start an explicitly discovered Terminal executable. Passive intent is provided
/// in STARTUPINFO before process creation and survives supported Terminal handoff.
/// The returned launcher PID is not a Terminal-window ownership proof.
pub fn start(
    plan: &LaunchPlan,
    executable: &Path,
    working_directory: &Path,
) -> Result<LauncherProcess, NativeError> {
    if !executable.is_absolute() || !executable.is_file() {
        return Err(NativeError::Invalid(
            "explicit installed Terminal executable required",
        ));
    }
    let application = wide(executable.as_os_str())?;
    let cwd = wide(working_directory.as_os_str())?;
    let arguments = std::iter::once(executable.to_string_lossy().into_owned())
        .chain(plan.arguments())
        .map(|argument| quote(&argument))
        .collect::<Vec<_>>()
        .join(" ");
    let mut command = wide(std::ffi::OsStr::new(&arguments))?;
    if command.len() > 32767 {
        return Err(NativeError::Invalid("command line too long"));
    }
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESHOWWINDOW,
        wShowWindow: if plan.intent == VisibleState::Passive {
            SW_SHOWNOACTIVATE as u16
        } else {
            SW_SHOWNORMAL as u16
        },
        ..unsafe { std::mem::zeroed() }
    };
    let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            0,
            0,
            null(),
            cwd.as_ptr(),
            &startup,
            &mut process,
        )
    } == 0
    {
        return Err(last("CreateProcessW Terminal"));
    }
    let process_handle = unsafe { OwnedHandle::from_raw_handle(process.hProcess) };
    let _thread = unsafe { OwnedHandle::from_raw_handle(process.hThread) };
    Ok(LauncherProcess {
        pid: process.dwProcessId,
        _process: process_handle,
    })
}

pub fn windows() -> Result<Vec<WindowCandidate>, NativeError> {
    unsafe extern "system" fn collect(hwnd: HWND, context: LPARAM) -> i32 {
        let windows = unsafe { &mut *(context as *mut Vec<WindowCandidate>) };
        let mut class = [0u16; 128];
        let length = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
        if length <= 0 {
            return 1;
        }
        let class = String::from_utf16_lossy(&class[..length as usize]);
        if class != "CASCADIA_HOSTING_WINDOW_CLASS" {
            return 1;
        }
        let length = unsafe { GetWindowTextLengthW(hwnd) };
        let mut title = vec![0u16; length.max(0) as usize + 1];
        let length = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32) };
        let mut pid = 0;
        if unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } == 0 {
            return 1;
        }
        windows.push(WindowCandidate {
            handle: hwnd as isize,
            process_id: pid,
            class,
            title: String::from_utf16_lossy(&title[..length.max(0) as usize]),
            existed_before_launch: false,
        });
        1
    }
    let mut windows = Vec::new();
    if unsafe { EnumWindows(Some(collect), &mut windows as *mut _ as LPARAM) } == 0 {
        return Err(last("EnumWindows Terminal candidates"));
    }
    Ok(windows)
}

pub struct ProcessIdentity {
    pub pid: u32,
    process: OwnedHandle,
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

pub struct VerifiedWindow {
    window: isize,
    owner: ProcessIdentity,
}
impl VerifiedWindow {
    pub fn window_handle(&self) -> isize {
        self.window
    }
    pub fn process_id(&self) -> u32 {
        self.owner.pid
    }
    /// Composition may pass these to its backend's explicit verified-launch bridge.
    pub fn into_parts(self) -> (isize, OwnedHandle) {
        (self.window, self.owner.process)
    }
}

pub fn verify(
    identity: &LaunchIdentity,
    proof: &SessionProof,
    before: &HashSet<isize>,
    terminal_executable: &Path,
) -> Result<VerifiedWindow, NativeError> {
    let mut candidates = windows()?;
    for candidate in &mut candidates {
        candidate.existed_before_launch = before.contains(&candidate.handle);
    }
    let mut chain = ancestry(proof.shell_pid)?;
    let ids: Vec<_> = chain.iter().map(|entry| entry.pid).collect();
    let candidate =
        correlate(identity, proof, &ids, &candidates).map_err(NativeError::Correlation)?;
    let index = chain
        .iter()
        .position(|owner| owner.pid == candidate.process_id)
        .ok_or(NativeError::Invalid(
            "window owner is outside session ancestry",
        ))?;
    let owner = chain.swap_remove(index);
    if !owner
        .executable
        .to_string_lossy()
        .eq_ignore_ascii_case(&terminal_executable.to_string_lossy())
    {
        return Err(NativeError::Invalid(
            "window executable does not match installed Terminal",
        ));
    }
    // Re-read the current candidate before handing its verified process/window to composition.
    if !windows()?.iter().any(|current| {
        current.handle == candidate.handle
            && current.process_id == owner.pid
            && current.title == identity.title()
    }) {
        return Err(NativeError::Invalid(
            "Terminal window changed during verification",
        ));
    }
    Ok(VerifiedWindow {
        window: candidate.handle,
        owner,
    })
}
