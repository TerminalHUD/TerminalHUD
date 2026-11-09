//! Exact retained CLI creation interval prevents adopting a concurrent owner.
use super::Result;
use std::{
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
    process::Child,
};
use windows_sys::Win32::{
    Foundation::*,
    System::{Diagnostics::ToolHelp::*, Threading::*},
};
fn times(handle: HANDLE) -> Result<(u64, u64)> {
    let (mut start, mut end, mut kernel, mut user) = unsafe {
        (
            std::mem::zeroed(),
            std::mem::zeroed(),
            std::mem::zeroed(),
            std::mem::zeroed(),
        )
    };
    if unsafe { GetProcessTimes(handle, &mut start, &mut end, &mut kernel, &mut user) } == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let number = |v: FILETIME| (u64::from(v.dwHighDateTime) << 32) | u64::from(v.dwLowDateTime);
    Ok((number(start), number(end)))
}
fn direct_parent(pid: u32) -> Result<u32> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error().into());
    }
    let _snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot) };
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let mut found = unsafe { Process32FirstW(snapshot, &mut entry) };
    while found != 0 {
        if entry.th32ProcessID == pid {
            return Ok(entry.th32ParentProcessID);
        }
        found = unsafe { Process32NextW(snapshot, &mut entry) };
    }
    Err("CLI-reported GUI is absent from process snapshot".into())
}
fn belongs_to_cli(parent: u32, cli: u32, created: u64, cli_created: u64, cli_exit: u64) -> bool {
    parent == cli && cli_exit != 0 && cli_created <= created && created <= cli_exit
}
pub(super) fn retain(pid: u32, child: &Child, cli: &Path) -> Result<OwnedHandle> {
    let process = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        )
    };
    if process.is_null() {
        return Err("cannot retain CLI-reported owner".into());
    }
    let process = unsafe { OwnedHandle::from_raw_handle(process) };
    let created = times(process.as_raw_handle())?.0;
    let (cli_created, cli_exit) = times(child.as_raw_handle())?;
    if !belongs_to_cli(
        direct_parent(pid)?,
        child.id(),
        created,
        cli_created,
        cli_exit,
    ) || unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } != WAIT_TIMEOUT
    {
        return Err(
            "CLI reply owner does not belong to retained startup interval; no cleanup command sent"
                .into(),
        );
    }
    let mut image = vec![0u16; 32768];
    let mut length = image.len() as u32;
    if unsafe {
        QueryFullProcessImageNameW(process.as_raw_handle(), 0, image.as_mut_ptr(), &mut length)
    } == 0
    {
        return Err("cannot inspect GUI image".into());
    }
    if !String::from_utf16_lossy(&image[..length as usize]).eq_ignore_ascii_case(
        &cli.with_file_name("terminalhud-shell.exe")
            .to_string_lossy(),
    ) {
        return Err("reported owner is not our GUI binary".into());
    }
    if terminalhud_app::windows::security::Identity::process(pid)?
        != terminalhud_app::windows::security::Identity::current()?
    {
        return Err("GUI user/session mismatch".into());
    }
    Ok(process)
}
#[test]
fn an_existing_or_replacement_gui_cannot_be_adopted_as_our_startup_owner() {
    assert!(belongs_to_cli(10, 10, 120, 100, 150));
    assert!(!belongs_to_cli(11, 10, 120, 100, 150));
    assert!(!belongs_to_cli(10, 10, 99, 100, 150));
    assert!(!belongs_to_cli(10, 10, 151, 100, 150));
    assert!(!belongs_to_cli(10, 10, 120, 100, 0));
}
