use super::{Process, attributes::Attributes, command_line, handles};
use crate::windows::native::{Handle, last};
use std::{ffi::OsString, fs::File, io, os::windows::ffi::OsStrExt, path::Path, ptr};
use windows_sys::Win32::System::Threading::*;
pub(crate) fn spawn(
    executable: &Path,
    args: &[OsString],
    diagnostic: &File,
) -> io::Result<Process> {
    let null = handles::null()?;
    let error = handles::diagnostic(diagnostic)?;
    let inherited = [null.raw(), error.raw()];
    let attributes = Attributes::new(&inherited)?;
    let mut command = command_line::arguments(executable, args)?;
    let application: Vec<_> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    if application[..application.len() - 1].contains(&0) {
        return Err(io::Error::other("NUL in GUI executable path"));
    }
    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = null.raw();
    startup.StartupInfo.hStdOutput = null.raw();
    startup.StartupInfo.hStdError = error.raw();
    startup.lpAttributeList = attributes.pointer;
    let mut information = unsafe { std::mem::zeroed() };
    // Keep normal direct parent, inherited environment/cwd/token and GUI
    // subsystem semantics. The list contains only our real inheritable handles.
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT,
            ptr::null(),
            ptr::null(),
            &startup.StartupInfo,
            &mut information,
        )
    } == 0
    {
        return Err(last("CreateProcessW GUI handle allowlist"));
    }
    let process = unsafe { Handle::from_raw(information.hProcess, "GUI process")? };
    let _thread = unsafe { Handle::from_raw(information.hThread, "GUI thread")? };
    Ok(Process {
        handle: process,
        pid: information.dwProcessId,
    })
}
