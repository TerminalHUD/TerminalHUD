//! Owned Win32 handles and native-call conversion mechanics.
use std::{
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
};
use windows_sys::Win32::Foundation::*;
pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub fn last(operation: &str) -> io::Error {
    io::Error::other(format!("{operation}: {}", io::Error::last_os_error()))
}

pub struct Handle(pub OwnedHandle);
impl Handle {
    /// # Safety
    /// A non-null, non-invalid input must be one uniquely owned, closeable Win32 handle.
    pub unsafe fn from_raw(raw: HANDLE, operation: &str) -> io::Result<Self> {
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            return Err(last(operation));
        }
        // The successful Win32 creation/open call transfers one owned handle.
        Ok(Self(unsafe { OwnedHandle::from_raw_handle(raw) }))
    }
    pub fn raw(&self) -> HANDLE {
        self.0.as_raw_handle()
    }
}
