//! Diagnostic-only process creation, isolated from guarded normal adapter APIs.
mod command;
use super::{Result, setup::Setup, show::Show};
use std::{
    os::windows::io::{FromRawHandle, OwnedHandle},
    ptr::null,
};
use terminalhud_windows_terminal::LaunchPlan;
use windows_sys::Win32::{Foundation::GetLastError, System::Threading::*};

pub(super) fn start(setup: &Setup, plan: &LaunchPlan, show: Show) -> Result<OwnedHandle> {
    let application = command::wide(setup.terminal.as_os_str())?;
    let cwd = command::wide(setup.task.as_os_str())?;
    let mut line = command::arguments(plan, &setup.terminal, &setup.command)?;
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESHOWWINDOW,
        wShowWindow: show.value(),
        ..unsafe { std::mem::zeroed() }
    };
    let mut process = unsafe { std::mem::zeroed::<PROCESS_INFORMATION>() };
    println!(
        "DIAGNOSTIC_CREATE_INTENT Passive STARTF_USESHOWWINDOW showCommand={} ({show:?})",
        show.value()
    );
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            line.as_mut_ptr(),
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
        return Err(format!("diagnostic CreateProcessW failed: {}", unsafe {
            GetLastError()
        })
        .into());
    }
    let process_handle = unsafe { OwnedHandle::from_raw_handle(process.hProcess) };
    let _thread = unsafe { OwnedHandle::from_raw_handle(process.hThread) };
    Ok(process_handle)
}
