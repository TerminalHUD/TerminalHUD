//! Explicit experiment choices; this type is private to the diagnostic example.
use windows_sys::Win32::UI::WindowsAndMessaging::{SW_SHOWNA, SW_SHOWNOACTIVATE};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Show {
    NoActivate,
    ShowNa,
}
impl Show {
    pub(crate) fn parse(argument: Option<&str>) -> Result<Self, &'static str> {
        match argument {
            Some("4") => Ok(Self::NoActivate),
            Some("8") => Ok(Self::ShowNa),
            _ => Err("diagnostic show command must be explicit 4 or 8"),
        }
    }
    pub(crate) fn value(self) -> u16 {
        match self {
            Self::NoActivate => SW_SHOWNOACTIVATE as u16,
            Self::ShowNa => SW_SHOWNA as u16,
        }
    }
}
