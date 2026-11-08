//! Query a retained process token without impersonating it.
use super::{
    super::native::{Handle, last},
    Identity,
};
use std::{io, mem, ptr};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{Authorization::ConvertSidToStringSidW, *},
    System::Threading::*,
};
pub(super) fn identity(pid: u32) -> io::Result<Identity> {
    // A retained process handle binds the token query to that process object.
    let process = unsafe {
        Handle::from_raw(
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid),
            "OpenProcess",
        )?
    };
    let mut raw = ptr::null_mut();
    if unsafe { OpenProcessToken(process.raw(), TOKEN_QUERY, &mut raw) } == 0 {
        return Err(last("OpenProcessToken"));
    }
    let token = unsafe { Handle::from_raw(raw, "OpenProcessToken")? };
    let mut size = 0;
    unsafe {
        GetTokenInformation(token.raw(), TokenUser, ptr::null_mut(), 0, &mut size);
    }
    if size == 0 {
        return Err(last("GetTokenInformation size"));
    }
    // usize storage gives TOKEN_USER and its in-buffer SID native alignment.
    let mut storage = vec![0usize; (size as usize).div_ceil(mem::size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token.raw(),
            TokenUser,
            storage.as_mut_ptr().cast(),
            size,
            &mut size,
        )
    } == 0
    {
        return Err(last("GetTokenInformation user"));
    }
    let user = unsafe { &*storage.as_ptr().cast::<TOKEN_USER>() };
    let mut text = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut text) } == 0 {
        return Err(last("ConvertSidToStringSidW"));
    }
    let mut len = 0;
    // ConvertSidToStringSidW returns a NUL-terminated LocalAlloc allocation.
    while unsafe { *text.add(len) } != 0 {
        len += 1;
    }
    let sid = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(text, len) });
    unsafe {
        LocalFree(text.cast());
    }
    let mut session = 0u32;
    if unsafe {
        GetTokenInformation(
            token.raw(),
            TokenSessionId,
            (&mut session as *mut u32).cast(),
            mem::size_of::<u32>() as u32,
            &mut size,
        )
    } == 0
    {
        return Err(last("GetTokenInformation session"));
    }
    Ok(Identity { sid, session })
}
