//! Contract between host configuration and the production launcher.
use terminalhud_core::config::Configuration;

#[expect(
    dead_code,
    reason = "production launcher supplies these effects during integration"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LaunchPreferenceEffect {
    Applied,
    RequiresRelaunch,
}

pub(crate) trait LaunchPreferences {
    fn prepare(
        &self,
        configuration: &Configuration,
    ) -> Result<Box<dyn PreparedLaunchPreferences>, String>;
}

pub(crate) trait PreparedLaunchPreferences {
    fn commit(self: Box<Self>) -> Result<LaunchPreferenceEffect, String>;
}
