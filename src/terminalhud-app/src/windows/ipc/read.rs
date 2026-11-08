use super::{super::native::last, IO_TIMEOUT, operation::Operation};
use std::io;
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};
pub(super) fn read(pipe: HANDLE, buffer: &mut [u8], stop: Option<HANDLE>) -> io::Result<usize> {
    let mut operation = Operation::new()?;
    let mut bytes = 0;
    if unsafe {
        ReadFile(
            pipe,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            &mut bytes,
            &mut operation.overlapped,
        )
    } != 0
    {
        return Ok(bytes as usize);
    }
    if unsafe { GetLastError() } != ERROR_IO_PENDING {
        return Err(last("ReadFile frame"));
    }
    operation.finish(pipe, stop, IO_TIMEOUT).map(|n| n as usize)
}
