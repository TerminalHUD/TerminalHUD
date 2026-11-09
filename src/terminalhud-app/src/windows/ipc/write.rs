use super::{super::native::last, IO_TIMEOUT, operation::Operation};
use std::io;
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};
pub(in crate::windows) fn write(
    pipe: HANDLE,
    buffer: &[u8],
    stop: Option<HANDLE>,
) -> io::Result<()> {
    let mut operation = Operation::new()?;
    let mut bytes = 0;
    let count = if unsafe {
        WriteFile(
            pipe,
            buffer.as_ptr(),
            buffer.len() as u32,
            &mut bytes,
            &mut operation.overlapped,
        )
    } != 0
    {
        bytes
    } else {
        if unsafe { GetLastError() } != ERROR_IO_PENDING {
            return Err(last("WriteFile frame"));
        }
        operation.finish(pipe, stop, IO_TIMEOUT)?
    };
    if count as usize != buffer.len() {
        return Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "partial pipe frame",
        ));
    }
    Ok(())
}
