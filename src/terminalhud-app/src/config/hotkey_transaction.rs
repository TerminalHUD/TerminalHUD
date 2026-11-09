//! Shortcut transaction shared by the Windows registration adapter and its fakes.
use terminalhud_core::{
    Command,
    config::{Configuration, Shortcut},
};

pub(crate) trait HotkeyPlatform {
    type Guard;
    fn register(
        &mut self,
        id: i32,
        shortcut: &Shortcut,
    ) -> Result<Self::Guard, RegisterError<Self::Guard>>;
    fn unregister(&mut self, guard: &mut Self::Guard) -> Result<(), String>;
}
pub(crate) struct RegisterError<G> {
    pub message: String,
    pub code: u32,
    pub remaining: Option<G>,
}
struct Entry<G> {
    shortcut: Shortcut,
    id: i32,
    guard: G,
}
pub(crate) struct Prepared<G>(Vec<Entry<G>>);
pub(crate) struct HotkeyTransaction<P: HotkeyPlatform> {
    platform: P,
    entries: Vec<Entry<P::Guard>>,
    stranded: Vec<(i32, P::Guard)>,
    actions: Vec<(i32, Command)>,
    pub(crate) error_code: u32,
}
impl<P: HotkeyPlatform> HotkeyTransaction<P> {
    pub(crate) fn new(platform: P) -> Self {
        Self {
            platform,
            entries: Vec::new(),
            stranded: Vec::new(),
            actions: Vec::new(),
            error_code: 0,
        }
    }
    pub(crate) fn prepare(&mut self, config: &Configuration) -> Result<Prepared<P::Guard>, String> {
        let mut prepared: Vec<Entry<P::Guard>> = Vec::new();
        for shortcut in [config.toggle_interaction(), config.toggle_visibility()] {
            if self.entries.iter().any(|entry| entry.shortcut == *shortcut) {
                continue;
            }
            let id = (0x5448..=0x54ff).find(|id| {
                !self.entries.iter().any(|e| e.id == *id)
                    && !prepared.iter().any(|e| e.id == *id)
                    && !self.stranded.iter().any(|(key, _)| key == id)
            });
            let Some(id) = id else {
                let cleanup = self.cancel(Prepared(prepared));
                return Err(format!("hotkey IDs exhausted; cleanup: {cleanup:?}"));
            };
            match self.platform.register(id, shortcut) {
                Ok(guard) => prepared.push(Entry {
                    shortcut: shortcut.clone(),
                    id,
                    guard,
                }),
                Err(error) => {
                    self.error_code = error.code;
                    if let Some(guard) = error.remaining {
                        self.stranded.push((id, guard));
                    }
                    let cleanup = self.cancel(Prepared(prepared));
                    return Err(format!(
                        "hotkey registration failed: {}; cleanup: {cleanup:?}",
                        error.message
                    ));
                }
            }
        }
        Ok(Prepared(prepared))
    }
    pub(crate) fn cancel(&mut self, prepared: Prepared<P::Guard>) -> Vec<String> {
        let mut failures = Vec::new();
        for mut entry in prepared.0 {
            if let Err(error) = self.platform.unregister(&mut entry.guard) {
                failures.push(error);
                self.stranded.push((entry.id, entry.guard));
            }
        }
        failures
    }
    pub(crate) fn commit(
        &mut self,
        prepared: Prepared<P::Guard>,
        config: &Configuration,
    ) -> Result<(), String> {
        self.entries.extend(prepared.0);
        self.actions = [
            (config.toggle_interaction(), Command::InteractionToggle),
            (config.toggle_visibility(), Command::VisibilityToggle),
        ]
        .into_iter()
        .map(|(shortcut, command)| {
            (
                self.entries
                    .iter()
                    .find(|e| e.shortcut == *shortcut)
                    .expect("prepared shortcut")
                    .id,
                command,
            )
        })
        .collect();
        let mut keep = Vec::new();
        let mut failures = Vec::new();
        for mut entry in self.entries.drain(..) {
            if config.toggle_interaction() == &entry.shortcut
                || config.toggle_visibility() == &entry.shortcut
            {
                keep.push(entry);
            } else if let Err(error) = self.platform.unregister(&mut entry.guard) {
                failures.push(error);
                self.stranded.push((entry.id, entry.guard));
            }
        }
        self.entries = keep;
        if failures.is_empty() {
            self.error_code = 0;
            Ok(())
        } else {
            Err(format!(
                "shortcut bindings changed, but old registrations could not be released: {failures:?}"
            ))
        }
    }
    pub(crate) fn actions(&self) -> &[(i32, Command)] {
        &self.actions
    }
}

#[cfg(test)]
#[path = "hotkey_transaction_tests.rs"]
mod tests;
