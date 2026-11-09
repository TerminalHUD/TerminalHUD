//! UI-thread-owned shortcut registrations and message routing.
use super::{UiMessage, UiSender, hotkey_binding::binding};
use crate::config::hotkey_transaction::{
    HotkeyPlatform, HotkeyTransaction, Prepared, RegisterError,
};
use hudswitch_windows::{HotkeyFailure, Hotkeys};
use std::cell::RefCell;
use terminalhud_core::{
    Command,
    config::{Configuration, Shortcut},
};

thread_local! { static ACTIONS: RefCell<Vec<(i32, Command)>> = const { RefCell::new(Vec::new()) }; }
pub(super) struct Native;
impl HotkeyPlatform for Native {
    type Guard = Hotkeys;
    fn register(
        &mut self,
        id: i32,
        shortcut: &Shortcut,
    ) -> Result<Hotkeys, RegisterError<Hotkeys>> {
        Hotkeys::register(&[binding(id, shortcut)]).map_err(|mut error| {
            let code = match &error.failure {
                HotkeyFailure::Win32 { code, .. } => *code,
                _ => u32::MAX,
            };
            let message = error.to_string();
            RegisterError {
                message,
                code,
                remaining: error.remaining.take(),
            }
        })
    }
    fn unregister(&mut self, guard: &mut Hotkeys) -> Result<(), String> {
        guard.unregister().map_err(|errors| format!("{errors:?}"))
    }
}
pub(super) struct Registrations(HotkeyTransaction<Native>);
impl Registrations {
    pub(super) fn new() -> Self {
        Self(HotkeyTransaction::new(Native))
    }
    pub(super) fn prepare(&mut self, config: &Configuration) -> Result<Prepared<Hotkeys>, String> {
        self.0.prepare(config)
    }
    pub(super) fn cancel(&mut self, prepared: Prepared<Hotkeys>) -> Vec<String> {
        self.0.cancel(prepared)
    }
    pub(super) fn commit(
        &mut self,
        prepared: Prepared<Hotkeys>,
        config: &Configuration,
    ) -> Result<(), String> {
        let result = self.0.commit(prepared, config);
        ACTIONS.with(|actions| *actions.borrow_mut() = self.0.actions().to_vec());
        result
    }
}
impl Drop for Registrations {
    fn drop(&mut self) {
        ACTIONS.with(|actions| actions.borrow_mut().clear());
    }
}
pub(super) fn register() -> (Registrations, u32) {
    let mut keys = Registrations::new();
    let config = Configuration::default();
    let error = keys
        .prepare(&config)
        .and_then(|prepared| keys.commit(prepared, &config))
        .err();
    if let Some(error) = error {
        eprintln!(
            "TerminalHUD shortcut registration failed: {error}; tray and IPC remain available"
        );
        let code = keys.0.error_code;
        (keys, if code == 0 { u32::MAX } else { code })
    } else {
        (keys, 0)
    }
}
pub(super) fn enqueue(id: usize, ui: &UiSender) -> bool {
    let command = ACTIONS.with(|actions| {
        actions
            .borrow()
            .iter()
            .find(|(key, _)| *key == id as i32)
            .map(|(_, command)| *command)
    });
    let Some(command) = command else {
        return false;
    };
    if let Err(error) = ui.send(UiMessage::Request {
        command,
        reply: None,
    }) {
        eprintln!("TerminalHUD hotkey command not queued: {error}");
    }
    true
}
#[cfg(test)]
pub(super) fn set_test_actions(actions: Vec<(i32, Command)>) {
    ACTIONS.with(|current| *current.borrow_mut() = actions);
}
#[cfg(test)]
#[path = "hotkey_tests.rs"]
mod tests;
