use super::super::native::{Handle, last};
use std::{io, mem, ptr};
use windows_sys::Win32::{
    Foundation::*,
    System::{IO::*, Threading::*},
};
pub(super) struct Operation {
    event: Handle,
    pub(super) overlapped: OVERLAPPED,
}
impl Operation {
    pub(super) fn new() -> io::Result<Self> {
        let event = unsafe {
            Handle::from_raw(
                CreateEventW(ptr::null(), 1, 0, ptr::null()),
                "CreateEvent IO",
            )?
        };
        let mut overlapped: OVERLAPPED = unsafe { mem::zeroed() };
        overlapped.hEvent = event.raw();
        Ok(Self { event, overlapped })
    }
    pub(super) fn finish(
        &mut self,
        pipe: HANDLE,
        stop: Option<HANDLE>,
        timeout: u32,
    ) -> io::Result<u32> {
        let handles = [self.event.raw(), stop.unwrap_or(self.event.raw())];
        let wait = unsafe {
            WaitForMultipleObjects(
                if stop.is_some() { 2 } else { 1 },
                handles.as_ptr(),
                0,
                timeout,
            )
        };
        if wait != WAIT_OBJECT_0 {
            // OVERLAPPED and its buffer remain alive until cancellation completes.
            unsafe {
                CancelIoEx(pipe, &self.overlapped);
            }
            let mut bytes = 0;
            unsafe {
                GetOverlappedResult(pipe, &self.overlapped, &mut bytes, 1);
            }
            return Err(io::Error::new(
                if wait == WAIT_TIMEOUT {
                    io::ErrorKind::TimedOut
                } else {
                    io::ErrorKind::Interrupted
                },
                "pipe operation timed out/stopped",
            ));
        }
        let mut bytes = 0;
        if unsafe { GetOverlappedResult(pipe, &self.overlapped, &mut bytes, 0) } == 0 {
            return Err(last("GetOverlappedResult"));
        }
        Ok(bytes)
    }
}
