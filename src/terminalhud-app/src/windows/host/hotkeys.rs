//! UI-thread-owned shortcut registrations. New chords are prepared before old ones are released.
use super::{UiMessage, UiSender, hotkey_binding::binding};
use hudswitch_windows::{HotkeyFailure, Hotkeys};
use std::cell::RefCell;
use terminalhud_core::{
    Command,
    config::{Configuration, Shortcut},
};

const FIRST_ID: i32 = 0x5448;
const LAST_ID: i32 = 0x54ff;
thread_local! { static ACTIONS: RefCell<Vec<(i32, Command)>> = const { RefCell::new(Vec::new()) }; }
struct Entry {
    shortcut: Shortcut,
    id: i32,
    guard: Hotkeys,
}
pub(super) struct Prepared(Vec<Entry>);

pub(super) struct Registrations {
    entries: Vec<Entry>,
    stranded: Vec<Hotkeys>,
    error_code: u32,
}
impl Registrations {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
            stranded: Vec::new(),
            error_code: 0,
        }
    }
    pub(super) fn prepare(&mut self, config: &Configuration) -> Result<Prepared, String> {
        let mut prepared = Vec::new();
        let existing: Vec<_> = self.entries.iter().map(|entry| &entry.shortcut).collect();
        for shortcut in new_shortcuts(&existing, config) {
            let id = (FIRST_ID..=LAST_ID)
                .find(|id| {
                    !self.entries.iter().any(|e| e.id == *id)
                        && !prepared.iter().any(|e: &Entry| e.id == *id)
                        && !self
                            .stranded
                            .iter()
                            .any(|g| g.bindings().iter().any(|b| b.id == *id))
                })
                .ok_or("hotkey IDs exhausted");
            let id = match id {
                Ok(id) => id,
                Err(error) => {
                    let cleanup = self.cancel(Prepared(prepared));
                    return Err(format!("{error}; cleanup: {cleanup:?}"));
                }
            };
            match Hotkeys::register(&[binding(id, &shortcut)]) {
                Ok(guard) => prepared.push(Entry {
                    shortcut,
                    id,
                    guard,
                }),
                Err(mut error) => {
                    self.error_code = match &error.failure {
                        HotkeyFailure::Win32 { code, .. } => *code,
                        _ => u32::MAX,
                    };
                    if let Some(guard) = error.remaining.take() {
                        self.stranded.push(guard);
                    }
                    let mut cleanup = Vec::new();
                    for mut entry in prepared {
                        if let Err(errors) = entry.guard.unregister() {
                            cleanup.extend(errors);
                            self.stranded.push(entry.guard);
                        }
                    }
                    return Err(format!(
                        "hotkey registration failed: {error}; cleanup failures: {cleanup:?}"
                    ));
                }
            }
        }
        Ok(Prepared(prepared))
    }
    pub(super) fn cancel(&mut self, prepared: Prepared) -> Vec<HotkeyFailure> {
        let mut failures = Vec::new();
        for mut entry in prepared.0 {
            if let Err(errors) = entry.guard.unregister() {
                failures.extend(errors);
                self.stranded.push(entry.guard);
            }
        }
        failures
    }
    pub(super) fn commit(
        &mut self,
        prepared: Prepared,
        config: &Configuration,
    ) -> Result<(), String> {
        let desired = [
            (config.toggle_interaction(), Command::InteractionToggle),
            (config.toggle_visibility(), Command::VisibilityToggle),
        ];
        self.entries.extend(prepared.0);
        let mut actions = Vec::new();
        for (shortcut, command) in desired {
            actions.push((
                self.entries
                    .iter()
                    .find(|e| e.shortcut == *shortcut)
                    .expect("prepared chord")
                    .id,
                command,
            ));
        }
        ACTIONS.with(|mapping| *mapping.borrow_mut() = actions);
        let mut failures = Vec::new();
        let mut keep = Vec::new();
        for mut entry in self.entries.drain(..) {
            if config.toggle_interaction() == &entry.shortcut
                || config.toggle_visibility() == &entry.shortcut
            {
                keep.push(entry);
            } else if let Err(errors) = entry.guard.unregister() {
                failures.extend(errors);
                self.stranded.push(entry.guard);
            }
        }
        self.entries = keep;
        if failures.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "hotkey bindings changed, but old registrations could not be released: {failures:?}"
            ))
        }
    }
}
fn new_shortcuts(existing: &[&Shortcut], config: &Configuration) -> Vec<Shortcut> {
    [config.toggle_interaction(), config.toggle_visibility()]
        .into_iter()
        .filter(|shortcut| !existing.contains(shortcut))
        .cloned()
        .collect()
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
        let code = keys.error_code;
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
