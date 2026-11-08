//! Portable TerminalHUD commands and serialized application-controller seam.

#![forbid(unsafe_code)]

use hudswitch_core::{Backend, Controller, Event, Status, TransitionError};

/// Platform composition imports these contracts without reversing crate edges.
pub mod integration {
    pub use hudswitch_core::{
        Acquired, Availability, Backend, Capabilities, Observation, Presentation, State, Status,
        VisibleState,
    };
}

/// Commands shared by user interfaces and local IPC, without transport policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    InteractionToggle,
    VisibilityToggle,
    Passive,
    Status,
    Quit,
}

#[derive(Debug)]
pub struct CommandOutcome<E> {
    pub status: Status,
    pub focus_restore_error: Option<E>,
    /// A host may exit only after confirmed restoration on an explicit quit.
    pub should_exit: bool,
}

/// Owned by the authoritative host thread; producers enqueue commands for it.
pub struct ShellController<B: Backend> {
    controller: Controller<B>,
}

impl<B: Backend> ShellController<B> {
    pub fn new(backend: B) -> Self {
        Self {
            controller: Controller::new(backend),
        }
    }

    pub fn status(&self) -> Status {
        self.controller.status()
    }

    pub fn execute(
        &mut self,
        command: Command,
    ) -> Result<CommandOutcome<B::Error>, TransitionError<B::Error>> {
        let event = match command {
            Command::Status => {
                return Ok(CommandOutcome {
                    status: self.status(),
                    focus_restore_error: None,
                    should_exit: false,
                });
            }
            Command::InteractionToggle => Event::InteractionToggle,
            Command::VisibilityToggle => Event::VisibilityToggle,
            Command::Passive => Event::RequestPassive,
            Command::Quit => Event::Quit,
        };
        let result = self.controller.dispatch(event)?;
        Ok(CommandOutcome {
            status: result.status,
            focus_restore_error: result.focus_restore_error,
            should_exit: command == Command::Quit,
        })
    }
}
