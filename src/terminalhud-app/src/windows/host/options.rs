use crate::mode::Mode;
use std::{path::PathBuf, time::Duration};
#[derive(Clone, Debug)]
pub struct HostOptions {
    pub instance: String,
    pub mode: Mode,
    pub development: Option<PathBuf>,
    pub headless: bool,
    pub menu_probe: bool,
    pub exit_after: Option<Duration>,
}
impl Default for HostOptions {
    fn default() -> Self {
        Self {
            instance: "default".into(),
            mode: Mode::Prototype,
            development: None,
            headless: false,
            menu_probe: false,
            exit_after: None,
        }
    }
}

impl HostOptions {
    pub(super) fn validate(&self) -> std::io::Result<()> {
        if self.mode == Mode::Prototype {
            crate::cli::validate_instance(&self.instance).map_err(std::io::Error::other)
        } else if self.instance != crate::mode::REAL_INSTANCE
            || self.headless
            || self.menu_probe
            || self.exit_after.is_some()
        {
            Err(std::io::Error::other(
                "real host uses its dedicated endpoint and no prototype test hooks",
            ))
        } else {
            Ok(())
        }
    }
}
