use super::super::native::wide;
use std::io;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub sid: String,
    pub session: u32,
}
impl Identity {
    pub fn current() -> io::Result<Self> {
        Self::process(unsafe { GetCurrentProcessId() })
    }
    pub fn process(pid: u32) -> io::Result<Self> {
        super::token::identity(pid)
    }

    pub fn pipe_name(&self, instance: &str) -> io::Result<Vec<u16>> {
        if instance.len() > 32
            || instance.is_empty()
            || !instance
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "prototype instance must be 1–32 ASCII letters/digits/-/_",
            ));
        }
        Ok(wide(&format!(
            r"\\.\pipe\TerminalHUD.prototype.v1.{}.{}.{}",
            self.sid, self.session, instance
        )))
    }
}
