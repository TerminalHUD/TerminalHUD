//! Receipt-bound, current-user Windows startup registration. Construction is read-only.
mod installed;
mod paths;
mod policy;
mod receipt;
mod registry;

use installed::Installed;
use policy::{RegistryValue, prepare};
use std::fmt;

#[derive(Debug)]
pub struct StartupError(String);
impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for StartupError {}
impl From<String> for StartupError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct StartupChange {
    expected: Option<RegistryValue>,
    desired: Option<RegistryValue>,
    command: String,
}
#[derive(Debug)]
pub struct StartupApplication {
    pub enabled: bool,
    pub changed: bool,
}
#[derive(Debug)]
pub struct StartupRegistration {
    installed: Installed,
}
impl StartupRegistration {
    pub fn for_current_install() -> Result<Self, StartupError> {
        Ok(Self {
            installed: Installed::current()?,
        })
    }
    /// Reads the receipt and Run value; never creates a key or writes a file.
    pub fn prepare(&self, enabled: bool) -> Result<StartupChange, StartupError> {
        let installed = self.installed.refresh()?;
        let expected = registry::read()?;
        let desired = prepare(
            &installed.command,
            &installed.owned_commands,
            expected.as_ref(),
            enabled,
        )?;
        Ok(StartupChange {
            expected,
            desired,
            command: installed.command,
        })
    }
    /// Revalidates ownership and expected state immediately before writing.
    /// Windows provides no atomic compare-and-swap for registry values.
    pub fn commit(&self, change: &StartupChange) -> Result<StartupApplication, StartupError> {
        let installed = self.installed.refresh()?;
        if installed.command != change.command {
            return Err(StartupError(
                "installation changed since startup preparation".into(),
            ));
        }
        let candidate = prepare(
            &installed.command,
            &installed.owned_commands,
            change.expected.as_ref(),
            change.desired.is_some(),
        )?;
        if candidate != change.desired {
            return Err(StartupError(
                "startup plan no longer matches installation".into(),
            ));
        }
        registry::apply(&change.expected, &change.desired)?;
        Ok(StartupApplication {
            enabled: change.desired.is_some(),
            changed: change.expected != change.desired,
        })
    }
}
