//! Enrollment of a single owned window with the long-lived session bridge.
use super::ipc::{accept, read, write};
mod client;
mod server;
const IO_TIMEOUT: u32 = 1500;
pub(super) use client::{completion, enroll, enroll_session, release};
pub(super) use server::{start, start_session};

fn creation_time(process: windows_sys::Win32::Foundation::HANDLE) -> std::io::Result<u64> {
    use windows_sys::Win32::{Foundation::FILETIME, System::Threading::GetProcessTimes};
    let mut created = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut exit = created;
    let mut kernel = created;
    let mut user = created;
    if unsafe { GetProcessTimes(process, &mut created, &mut exit, &mut kernel, &mut user) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod session_tests;
