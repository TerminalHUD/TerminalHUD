//! Read-only supported native provider discovery.
mod powershell;
mod wsl;
use super::NativeError;
use crate::GeneratedProfile;
pub(super) use wsl::fragment_guid;

pub fn discover_generated_profiles() -> Result<Vec<GeneratedProfile<'static>>, NativeError> {
    let mut profiles = powershell::discover()?;
    profiles.extend(wsl::discover()?);
    Ok(profiles)
}
