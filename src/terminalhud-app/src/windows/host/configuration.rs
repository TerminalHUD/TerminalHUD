use super::{context::HostContext, launch_preferences::LaunchPreferenceEffect};
use crate::{
    config,
    protocol::{Reply, ReplyCode},
};
use std::sync::mpsc;
use terminalhud_core::config::Configuration;

impl HostContext {
    fn report_configuration(&self, result: Result<String, String>) -> bool {
        match result {
            Ok(message) => {
                eprintln!("TerminalHUD: {message}");
                *self.configuration_error.borrow_mut() = None;
                if let Some(tray) = self.tray.borrow().as_ref() {
                    let _ = tray.set_tooltip(Some("TerminalHUD"));
                }
                true
            }
            Err(error) => {
                eprintln!("TerminalHUD configuration: {error}");
                if let Some(tray) = self.tray.borrow().as_ref() {
                    let detail: String = error
                        .chars()
                        .rev()
                        .take(50)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    let _ = tray.set_tooltip(Some(format!("TerminalHUD configuration: {detail}")));
                }
                *self.configuration_error.borrow_mut() = Some(error);
                false
            }
        }
    }
    pub(super) fn open_configuration(&self) {
        if self.quitting.get() {
            return;
        }
        let result = config::settings_path().and_then(|path| {
            config::ensure_file(&path)?;
            config::open_document(&path)?;
            Ok(format!("Opened {}", path.display()))
        });
        // Opening a file does not establish that a previous error was corrected.
        if let Err(error) = result {
            self.report_configuration(Err(error));
        }
    }
    pub(super) fn validate_configuration(&self) {
        if self.quitting.get() {
            return;
        }
        let result = config::settings_path().and_then(|path| {
            config::validate_file(&path)?;
            Ok(format!("Valid configuration: {}", path.display()))
        });
        match result {
            Ok(message) => eprintln!("TerminalHUD: {message}"),
            Err(error) => {
                self.report_configuration(Err(error));
            }
        }
    }
    pub(super) fn reload_configuration(&self, reply: Option<mpsc::SyncSender<Reply>>) {
        if self.quitting.get() {
            if let Some(sender) = reply {
                let _ = sender.send(self.reply(ReplyCode::ShuttingDown, false));
            }
            return;
        }
        self.sequence.set(self.sequence.get().saturating_add(1));
        if reply.is_none() {
            self.menu_commands
                .set(self.menu_commands.get().saturating_add(1));
        }
        let initial = self.requires_configuration && self.configuration.borrow().is_none();
        let code = match self.apply_configuration() {
            Ok(effect) => {
                if initial {
                    self.initialize_real();
                }
                let (code, message) = match effect {
                    LaunchPreferenceEffect::Applied => (ReplyCode::Ok, "Configuration applied"),
                    LaunchPreferenceEffect::RequiresRelaunch => (
                        ReplyCode::RequiresRelaunch,
                        "Configuration applied; terminal appearance changes require the next launch",
                    ),
                };
                self.report_configuration(Ok(message.into()));
                if effect == LaunchPreferenceEffect::RequiresRelaunch
                    && let Some(tray) = self.tray.borrow().as_ref()
                {
                    let _ = tray
                        .set_tooltip(Some("TerminalHUD — terminal changes apply on next launch"));
                }
                code
            }
            Err(error) => {
                self.report_configuration(Err(error));
                ReplyCode::TransitionFailed
            }
        };
        if let Some(sender) = reply {
            let _ = sender.send(self.reply(code, false));
        }
    }
    pub(super) fn initialize_configuration(&self) -> bool {
        let result = config::settings_path()
            .and_then(|path| config::load_or_create(&path))
            .and_then(|candidate| {
                let launch = self
                    .launch_preferences
                    .as_ref()
                    .ok_or("production launch preferences are unavailable")?;
                self.apply_candidate(candidate, launch.as_ref())
            });
        self.report_configuration(result.map(|_| "Startup configuration applied".into()))
    }
    fn apply_configuration(&self) -> Result<LaunchPreferenceEffect, String> {
        let launch = self
            .launch_preferences
            .as_ref()
            .ok_or("configuration reload unavailable for this host")?;
        let path = config::settings_path()?;
        let candidate = config::validate_file(&path)?;
        self.apply_candidate(candidate, launch.as_ref())
    }
    fn apply_candidate(
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
        let prepared_keys = match hotkeys.prepare(&candidate) {
            Ok(prepared) => prepared,
            Err(error) => {
                self.hotkey_error.set(hotkeys.error_code().max(1));
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
        *self.configuration.borrow_mut() = Some(candidate);
        Ok(effect)
    }
}
