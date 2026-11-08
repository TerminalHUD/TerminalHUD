use super::command::{arguments, wide};
use super::{NativeError, error::last};
use crate::LaunchPlan;
use hudswitch_core::VisibleState;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::path::Path;
use std::ptr::null;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
pub struct LauncherProcess {
    pub pid: u32,
    _process: OwnedHandle,
}

/// Start an explicitly discovered Terminal executable. Normal Passive fullscreen
/// requests return `Unsupported` before spawning: the measured release activates
/// transiently. A returned launcher PID never proves window ownership.
///
/// Precondition: independently verify the runtime profile, or establish a fresh
/// isolated configuration whose profile AND fallback command/connection/elevation
/// are safe. A fragment file on disk alone is insufficient: an unknown GUID may
/// fall back to an ordinary profile with unsafe launch behavior. The test probe
/// uses the controlled portable precondition, not a runtime-recognition claim.
pub fn start(
    plan: &LaunchPlan,
    executable: &Path,
    working_directory: &Path,
) -> Result<LauncherProcess, NativeError> {
    super::support::run(plan, || create(plan, executable, working_directory, &[]))
}

/// Supply an explicit replacement command for a controlled launch. This does
/// not by itself neutralize inherited elevation/connection behavior: the caller
/// must still establish the profile/default-settings precondition of `start`.
/// Terminal command separators are rejected instead of opening extra sessions.
pub fn start_with_command(
    plan: &LaunchPlan,
    executable: &Path,
    working_directory: &Path,
    command: &[String],
) -> Result<LauncherProcess, NativeError> {
    super::support::run(plan, || {
        if command.first().is_none_or(|argument| argument.is_empty()) {
            return Err(NativeError::Invalid("explicit launch command is required"));
        }
        create(plan, executable, working_directory, command)
    })
}

fn create(
    plan: &LaunchPlan,
    executable: &Path,
    working_directory: &Path,
    command: &[String],
) -> Result<LauncherProcess, NativeError> {
    if !executable.is_absolute() || !executable.is_file() {
        return Err(NativeError::Invalid(
            "explicit installed Terminal executable required",
        ));
    }
    let application = wide(executable.as_os_str())?;
    let cwd = wide(working_directory.as_os_str())?;
    let mut command = arguments(plan, executable, command)?;
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
