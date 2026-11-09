//! Contract between host configuration and the production launcher.
use terminalhud_core::config::Configuration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LaunchPreferenceEffect {
    Applied,
    RequiresRelaunch,
}

pub(crate) trait LaunchPreferences {
    fn prepare(
        &self,
        configuration: &Configuration,
        target_present: bool,
    ) -> Result<Box<dyn PreparedLaunchPreferences>, String>;
}

pub(crate) trait PreparedLaunchPreferences {
    fn commit(self: Box<Self>) -> Result<LaunchPreferenceEffect, String>;
}
