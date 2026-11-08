use super::{NativeError, error::last};
use crate::LaunchPlan;
use hudswitch_core::VisibleState;
use std::os::windows::{
    ffi::OsStrExt,
    io::{FromRawHandle, OwnedHandle},
};
use std::path::Path;
use std::ptr::null;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
fn wide(value: &std::ffi::OsStr) -> Result<Vec<u16>, NativeError> {
    let mut value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err(NativeError::Invalid("NUL in native path/argument"));
    }
    value.push(0);
    Ok(value)
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
///
/// Precondition: the caller has independently verified that Terminal's runtime
/// catalog contains this profile. A file on disk is insufficient: an unknown
/// GUID may fall back to an ordinary profile, including its elevation/connection
/// settings. The unfinished proof deliberately does not call this operation.
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
