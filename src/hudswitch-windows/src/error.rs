use std::fmt;
use windows_sys::Win32::Foundation::GetLastError;

#[derive(Debug)]
pub enum Error {
    Win32 { operation: &'static str, code: u32 },
    Launch(std::io::Error),
    InvalidTarget,
    AlreadyManaged,
    Unsupported(&'static str),
    FocusRefused,
    Timeout(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

pub(crate) fn last_error(operation: &'static str) -> Error {
    Error::Win32 {
        operation,
        code: unsafe { GetLastError() },
    }
}
