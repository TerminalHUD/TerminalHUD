use super::{context::HostContext, launch_preferences::LaunchPreferenceEffect};
use crate::config;
use terminalhud_core::config::Configuration;

impl HostContext {
    pub(super) fn apply_configuration(&self) -> Result<LaunchPreferenceEffect, String> {
        let launch = self
            .launch_preferences
            .as_ref()
            .ok_or("configuration reload unavailable for this host")?;
        let path = config::settings_path()?;
        let candidate = config::validate_file(&path)?;
        self.apply_candidate(candidate, launch.as_ref())
    }
    pub(super) fn apply_candidate(
        &self,
        candidate: Configuration,
        launch: &dyn super::launch_preferences::LaunchPreferences,
    ) -> Result<LaunchPreferenceEffect, String> {
        if self.startup_registration.borrow().is_none() {
            let startup = crate::windows::startup::StartupRegistration::for_current_install()
                .map_err(|error| format!("startup registration unavailable: {error}"))?;
            *self.startup_registration.borrow_mut() = Some(startup);
        }
        let startup = self.startup_registration.borrow();
        let startup = startup.as_ref().expect("startup registration initialized");
        let startup_change = startup
            .prepare(candidate.start_with_windows())
            .map_err(|error| format!("startup registration preparation failed: {error}"))?;
        let prepared_launch = launch.prepare(&candidate)?;
        let mut hotkeys = self.hotkeys.borrow_mut();
        let hotkeys = hotkeys.as_mut().ok_or("hotkey service unavailable")?;
        let bootstrap = self.requires_configuration && !self.launch_ready.get();
        let prepared_keys = match hotkeys.prepare(&candidate) {
            Ok(prepared) => prepared,
            Err(error) => {
                self.hotkey_error.set(hotkeys.error_code().max(1));
                if bootstrap {
                    config::bootstrap::apply_without_shortcuts(
                        &error,
                        || {
                            startup
                                .commit(&startup_change)
                                .map(|applied| applied.changed)
                                .map_err(|failure| failure.to_string())
                        },
                        || prepared_launch.commit().map(|_| ()),
                    )?;
                    self.launch_ready.set(true);
                    return Err(format!(
                        "{error}; startup and launch preferences applied; shortcuts unavailable; use tray or IPC"
                    ));
                }
                return Err(error);
            }
        };
        let startup_applied = match startup.commit(&startup_change) {
            Ok(applied) => applied,
            Err(error) => {
                let cleanup = hotkeys.cancel(prepared_keys);
                return Err(format!(
                    "startup registration commit failed: {error}; hotkey cleanup: {cleanup:?}; inspect startup registration"
                ));
            }
        };
        let effect = match prepared_launch.commit() {
            Ok(effect) => effect,
            Err(error) => {
                let cleanup = hotkeys.cancel(prepared_keys);
                return Err(format!(
                    "launch preference commit failed: {error}; startup registration committed (changed={}); hotkey cleanup: {cleanup:?}; inspect runtime state",
                    startup_applied.changed
                ));
            }
        };
        if let Err(error) = hotkeys.commit(prepared_keys, &candidate) {
            self.hotkey_error.set(u32::MAX);
            return Err(format!(
                "{error}; startup registration and launch preferences have changed; inspect runtime state"
            ));
        }
        self.hotkey_error.set(0);
        self.launch_ready.set(true);
        *self.configuration.borrow_mut() = Some(candidate);
        Ok(effect)
    }
}
