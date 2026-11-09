use super::context::HostContext;
use terminalhud_core::Command;
impl HostContext {
    pub(super) fn initialize_real(&self) {
        // Setup is complete before the first target mutation. A launch failure
        // keeps its recovery state owned by this tray and readiness reports it.
        let already_busy = self.busy.replace(true);
        if let Err(error) = self
            .controller
            .borrow_mut()
            .execute(Command::InteractionToggle)
        {
            eprintln!(
                "TerminalHUD initial Active creation failed: {error}; tray remains available"
            );
            *self.last_command_error.borrow_mut() = Some(error);
        }
        self.busy.set(already_busy);
    }
}
