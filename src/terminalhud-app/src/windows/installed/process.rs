//! Preserve the selected raw Windows command line and inherited Terminal context.
use std::{
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    ptr,
};
use windows_sys::Win32::{
    Foundation::*,
    System::{
        Environment::ExpandEnvironmentStringsW, SystemInformation::GetSystemDirectoryW,
        Threading::*,
    },
};

pub(super) struct Shell(OwnedHandle);
impl Shell {
    pub(super) fn wait(self) -> io::Result<u32> {
        if unsafe { WaitForSingleObject(self.0.as_raw_handle(), INFINITE) } != WAIT_OBJECT_0 {
            return Err(io::Error::last_os_error());
        }
        let mut code = 0;
        if unsafe { GetExitCodeProcess(self.0.as_raw_handle(), &mut code) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(code)
    }
}
fn expand(value: &str) -> io::Result<String> {
    if value.contains('\0') {
        return Err(io::Error::other("NUL in profile launch context"));
    }
    let source: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let size = unsafe { ExpandEnvironmentStringsW(source.as_ptr(), ptr::null_mut(), 0) };
    if size == 0 || size > 32767 {
        return Err(io::Error::other(
            "profile launch context cannot be expanded",
        ));
    }
    let mut output = vec![0u16; size as usize];
    let count = unsafe { ExpandEnvironmentStringsW(source.as_ptr(), output.as_mut_ptr(), size) };
    if count == 0 || count > size {
        return Err(io::Error::other(
            "profile environment changed during expansion",
        ));
    }
    String::from_utf16(&output[..count as usize - 1]).map_err(io::Error::other)
}
fn system_directory() -> io::Result<String> {
    let mut output = vec![0u16; 32768];
    let count = unsafe { GetSystemDirectoryW(output.as_mut_ptr(), output.len() as u32) } as usize;
    if count == 0 || count >= output.len() {
        return Err(io::Error::other("Windows system directory unavailable"));
    }
    String::from_utf16(&output[..count]).map_err(io::Error::other)
}
pub(super) fn start(command: &str, directory: &str) -> io::Result<Shell> {
    if command.trim().is_empty() {
        return Err(io::Error::other("selected profile has no command line"));
    }
    let command = expand(command)?;
    let directory = expand(directory)?;
    let command =
        terminalhud_windows_terminal::plan_wsl_launch(&command, &directory, &system_directory()?)
            .map_or(command, |plan| plan.commandline);
    let mut command: Vec<u16> = command.encode_utf16().chain(Some(0)).collect();
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe {
        CreateProcessW(
            ptr::null(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            1,
            0,
            ptr::null(),
            ptr::null(),
            &startup,
            &mut process,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let process_handle = unsafe { OwnedHandle::from_raw_handle(process.hProcess) };
    let _thread = unsafe { OwnedHandle::from_raw_handle(process.hThread) };
    Ok(Shell(process_handle))
}

#[cfg(test)]
mod tests;
