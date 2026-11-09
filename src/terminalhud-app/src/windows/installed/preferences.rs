//! One shared next-launch snapshot for configuration and the real launcher.
use super::setup;
use crate::windows::host::launch_preferences::{
    LaunchPreferenceEffect, LaunchPreferences, PreparedLaunchPreferences,
};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use terminalhud_core::config::Configuration;
use terminalhud_windows_terminal::Guid;

#[derive(Clone, PartialEq)]
pub(super) struct Snapshot {
    pub terminal: PathBuf,
    pub profile: Guid,
    pub commandline: String,
    pub starting_directory: Option<String>,
    pub appearance: serde_json::Value,
    pub config: Configuration,
}
#[derive(Clone, Default)]
pub(in crate::windows) struct Shared(Rc<RefCell<Option<Snapshot>>>);
impl Shared {
    pub(super) fn snapshot(&self) -> Result<Snapshot, String> {
        self.0.borrow().clone().ok_or_else(|| "Run `terminalhud profiles plan`, then `terminalhud profiles apply` to set up the installed Terminal profile".into())
    }
    pub(super) fn refresh(&self) -> Result<Snapshot, String> {
        let config = self.snapshot()?.config;
        let plan = setup::prepare(&config).map_err(|error| error.to_string())?;
        if !plan.is_owned() {
            return Err(
                "TerminalHUD profile ownership is missing; review `terminalhud profiles plan`"
                    .into(),
            );
        }
        plan.commit().map_err(|error| error.to_string())?;
        let snapshot = snapshot(&plan, config);
        *self.0.borrow_mut() = Some(snapshot.clone());
        Ok(snapshot)
    }
}
fn snapshot(plan: &setup::Plan, config: Configuration) -> Snapshot {
    Snapshot {
        terminal: plan.sources.executable.clone(),
        profile: plan.profile.hud_guid.clone(),
        commandline: plan.profile.commandline.clone(),
        starting_directory: plan.profile.starting_directory.clone(),
        appearance: plan.profile.owned_profile.clone(),
        config,
    }
}
struct Prepared {
    shared: Shared,
    plan: setup::Plan,
    config: Configuration,
    target_present: bool,
}
impl LaunchPreferences for Shared {
    fn prepare(
        &self,
        config: &Configuration,
        target_present: bool,
    ) -> Result<Box<dyn PreparedLaunchPreferences>, String> {
        let plan = setup::prepare(config).map_err(|error| error.to_string())?;
        if !plan.is_owned() {
            return Err("Run `terminalhud profiles plan`, then `terminalhud profiles apply` to set up the installed Terminal profile".into());
        }
        Ok(Box::new(Prepared {
            shared: self.clone(),
            plan,
            config: config.clone(),
            target_present,
        }))
    }
}
impl PreparedLaunchPreferences for Prepared {
    fn commit(self: Box<Self>) -> Result<LaunchPreferenceEffect, String> {
        self.plan.commit().map_err(|error| error.to_string())?;
        let next = snapshot(&self.plan, self.config);
        let effect = effect(self.shared.0.borrow().as_ref(), &next, self.target_present);
        *self.shared.0.borrow_mut() = Some(next);
        Ok(effect)
    }
}

fn effect(
    previous: Option<&Snapshot>,
    next: &Snapshot,
    target_present: bool,
) -> LaunchPreferenceEffect {
    let changed = previous.is_some_and(|old| {
        old.terminal != next.terminal
            || old.profile != next.profile
            || old.commandline != next.commandline
            || old.starting_directory != next.starting_directory
            || old.appearance != next.appearance
    });
    if changed && target_present {
        LaunchPreferenceEffect::RequiresRelaunch
    } else {
        LaunchPreferenceEffect::Applied
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_a_changed_live_launch_requires_relaunch() {
        let old = Snapshot {
            terminal: "terminal.exe".into(),
            profile: Guid::parse("{11111111-1111-1111-1111-111111111111}").unwrap(),
            commandline: "shell".into(),
            starting_directory: None,
            appearance: serde_json::json!({"opacity":35}),
            config: Configuration::default(),
        };
        let mut next = old.clone();
        assert_eq!(
            effect(Some(&old), &next, true),
            LaunchPreferenceEffect::Applied
        );
        next.appearance = serde_json::json!({"opacity":60});
        assert_eq!(
            effect(Some(&old), &next, true),
            LaunchPreferenceEffect::RequiresRelaunch
        );
        assert_eq!(
            effect(Some(&old), &next, false),
            LaunchPreferenceEffect::Applied
        );
    }
}
