use super::{Controller, Managed};
use crate::*;

impl<B: Backend> Controller<B> {
    /// Complete one transition before accepting the next event.
    pub fn dispatch(
        &mut self,
        event: Event,
    ) -> Result<Outcome<B::Error>, TransitionError<B::Error>> {
        let mut target = self.managed.take();
        let result = self.dispatch_inner(&mut target, event);
        self.managed = target;
        result.map(|warning| Outcome {
            status: self.status(),
            focus_restore_error: warning,
        })
    }

    fn dispatch_inner(
        &mut self,
        target: &mut Option<Managed<B>>,
        event: Event,
    ) -> Result<Option<B::Error>, TransitionError<B::Error>> {
        if self.stopped {
            return if event == Event::Quit {
                Ok(None)
            } else {
                Err(Self::error(Stage::Validate, Cause::Stopped))
            };
        }
        if let Event::TargetClosed(id) = event {
            if target.as_ref().is_some_and(|current| current.id == id) {
                *target = None;
            }
            return Ok(None);
        }
        if let Event::FocusLost(id) = event
            && !target.as_ref().is_some_and(|current| current.id == id)
        {
            return Ok(None);
        }
        self.validate_target(target)?;
        if event == Event::Quit {
            return self.shutdown(target);
        }
        let state = target.as_ref().and_then(|current| current.state);
        if target.is_some() && state.is_none() {
            let mut failure = Self::error(Stage::Observe, Cause::RecoveryRequired);
            failure.recovery = Recovery::Unconfirmed;
            return Err(failure);
        }
        let Some(requested) = super::transition::requested_state(event, state) else {
            return Ok(None);
        };
        self.check_capabilities(requested, target.is_none())?;
        // Capture the external foreground before acquisition can activate a new window.
        let focus = if requested == State::Active {
            self.backend
                .foreground()
                .map_err(|error| Self::error(Stage::CaptureFocus, Cause::Backend(error)))?
        } else {
            None
        };
        if target.is_none() {
            self.acquire(target, requested)?;
        }
        let current = target.as_mut().expect("target was acquired");
        if requested == State::Active {
            current.focus = focus;
        }
        let preserve_foreground = matches!(event, Event::FocusLost(_))
            || (requested != State::Active && state != Some(State::Active));
        self.apply(current, requested, preserve_foreground)
            .map_err(|failure| {
                self.failed(target, failure.stage, failure.cause, preserve_foreground)
            })
    }
}
