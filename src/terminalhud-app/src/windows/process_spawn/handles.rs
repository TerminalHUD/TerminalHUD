use crate::windows::native::{Handle, last, wide};
use std::{fs::File, io, os::windows::io::AsRawHandle, ptr};
use windows_sys::Win32::{
    Foundation::*, Security::SECURITY_ATTRIBUTES, Storage::FileSystem::*,
    System::Threading::GetCurrentProcess,
};
pub(super) fn null() -> io::Result<Handle> {
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: 1,
    };
    unsafe {
        Handle::from_raw(
            CreateFileW(
                wide("NUL").as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                &attributes,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            ),
            "CreateFile GUI NUL",
        )
    }
}
pub(super) fn diagnostic(file: &File) -> io::Result<Handle> {
    let mut output = ptr::null_mut();
    let current = unsafe { GetCurrentProcess() };
    // Only this owned duplicate becomes inheritable; original/capture handles
    // retain their flags. Pseudo handles are API parameters, never list entries.
    if unsafe {
        DuplicateHandle(
            current,
            file.as_raw_handle(),
            current,
            &mut output,
            0,
            1,
            DUPLICATE_SAME_ACCESS,
        )
    } == 0
    {
        return Err(last("DuplicateHandle GUI diagnostic"));
    }
    unsafe { Handle::from_raw(output, "duplicate GUI diagnostic") }
}
