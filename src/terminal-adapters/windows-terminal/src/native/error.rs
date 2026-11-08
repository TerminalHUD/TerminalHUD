use std::fmt;
use windows_sys::Win32::Foundation::GetLastError;
#[derive(Debug)]
pub enum NativeError {
    Win32(&'static str, u32),
    Invalid(&'static str),
    Correlation(crate::Error),
    AncestryReused { child_pid: u32, parent_pid: u32 },
}
impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NativeError {}
pub(super) fn last(operation: &'static str) -> NativeError {
    NativeError::Win32(operation, unsafe { GetLastError() })
}
