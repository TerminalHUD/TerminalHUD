use super::{super::native::last, operation::Operation};
use std::io;
use windows_sys::Win32::{
    Foundation::*,
    System::{Pipes::*, Threading::INFINITE},
};
pub(in crate::windows) fn accept(pipe: HANDLE, stop: HANDLE) -> io::Result<()> {
    let mut operation = Operation::new()?;
    if unsafe { ConnectNamedPipe(pipe, &mut operation.overlapped) } != 0 {
        return Ok(());
    }
    match unsafe { GetLastError() } {
        ERROR_PIPE_CONNECTED => Ok(()),
        ERROR_IO_PENDING => operation.finish(pipe, Some(stop), INFINITE).map(|_| ()),
        _ => Err(last("ConnectNamedPipe")),
    }
}
