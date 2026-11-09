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
                    let _ = tray.set_tooltip(Some(
                        "TerminalHUD — configuration error; inspect configuration",
                    ));
                }
                *self.configuration_error.borrow_mut() = Some(error);
                false
            }
        }
    }
    pub(super) fn open_configuration(&self) {
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
        let result = self.apply_configuration();
        let code = if self.report_configuration(result) {
            ReplyCode::Ok
        } else {
            ReplyCode::TransitionFailed
        };
        if let Some(sender) = reply {
            let _ = sender.send(self.reply(code, false));
        }
    }
    fn apply_configuration(&self) -> Result<String, String> {
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
    ) -> Result<String, String> {
        let previous = self.configuration.borrow();
        if previous
            .as_ref()
            .is_none_or(|old| old.start_with_windows() != candidate.start_with_windows())
        {
            return Err(
                "startup registration service is unavailable; configuration was not applied".into(),
            );
        }
        let prepared_launch = launch.prepare(&candidate)?;
        let mut hotkeys = self.hotkeys.borrow_mut();
        let hotkeys = hotkeys.as_mut().ok_or("hotkey service unavailable")?;
        let prepared_keys = hotkeys.prepare(&candidate)?;
        let effect = match prepared_launch.commit() {
            Ok(effect) => effect,
            Err(error) => {
                let cleanup = hotkeys.cancel(prepared_keys);
                return Err(format!(
                    "launch preference commit failed: {error}; hotkey cleanup: {cleanup:?}; inspect runtime state"
                ));
            }
        };
        if let Err(error) = hotkeys.commit(prepared_keys, &candidate) {
            return Err(format!(
                "{error}; launch preferences have changed; inspect runtime state"
            ));
        }
        drop(previous);
        *self.configuration.borrow_mut() = Some(candidate);
        Ok(match effect {
            LaunchPreferenceEffect::Applied => "Configuration applied".into(),
            LaunchPreferenceEffect::RequiresRelaunch => {
                "Configuration applied; terminal appearance changes require the next launch".into()
            }
        })
    }
}
