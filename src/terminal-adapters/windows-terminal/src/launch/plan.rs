use crate::Guid;
use hudswitch_core::VisibleState;
/// A unique launch correlation. Use fresh OS-generated GUIDs for actual launches.
#[derive(Clone, Debug)]
pub struct LaunchIdentity {
    pub nonce: Guid,
    pub profile: Guid,
}
impl LaunchIdentity {
    pub fn window_name(&self) -> String {
        format!("TerminalHUD-{}", self.nonce.bare())
    }
    pub fn title(&self) -> String {
        format!("TerminalHUD {}", self.nonce.bare())
    }
}

#[derive(Clone, Debug)]
pub struct LaunchPlan {
    pub identity: LaunchIdentity,
    pub intent: VisibleState,
    pub primary_origin: (i32, i32),
}
impl LaunchPlan {
    /// Keep the selected profile's environment policy when an explicit bridge
    /// command would otherwise make Terminal implicitly inherit its environment.
    /// Append the replacement command after these new-tab options.
    pub fn arguments_with_environment(&self, reload_environment: bool) -> Vec<String> {
        let mut arguments = self.arguments();
        arguments.push(
            if reload_environment {
                "--reloadEnvironment"
            } else {
                "--inheritEnvironment"
            }
            .into(),
        );
        arguments
    }

    pub fn arguments(&self) -> Vec<String> {
        vec![
            "--window".into(),
            self.identity.window_name(),
            "--fullscreen".into(),
            "--pos".into(),
            format!("{},{}", self.primary_origin.0, self.primary_origin.1),
            "new-tab".into(),
            "--profile".into(),
            self.identity.profile.to_string(),
            "--title".into(),
            self.identity.title(),
            "--suppressApplicationTitle".into(),
        ]
    }
}
