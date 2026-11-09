use super::{context::HostContext, launch_preferences::LaunchPreferenceEffect};
use crate::{
    config,
    protocol::{Reply, ReplyCode},
};
use std::sync::mpsc;

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
        let initial = self.requires_configuration && !self.launch_ready.get();
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
        self.report_configuration(result.map(|_| "Startup configuration applied".into()));
        self.launch_ready.get()
    }
}
