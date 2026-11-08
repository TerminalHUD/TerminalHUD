use super::{Controller, Managed};
use crate::{Backend, Cause, Recovery, Stage, State, TransitionError};

impl<B: Backend> Controller<B> {
    pub(super) fn validate_target(
        &mut self,
        target: &mut Option<Managed<B>>,
    ) -> Result<(), TransitionError<B::Error>> {
        if let Some(current) = target.as_mut() {
            match self.backend.is_valid(&current.target) {
                Ok(false) => *target = None,
                Ok(true) => {}
                Err(error) => {
                    current.state = None;
                    let mut failure = Self::error(Stage::Validate, Cause::Backend(error));
                    failure.recovery = Recovery::Unconfirmed;
                    return Err(failure);
                }
            }
        }
        Ok(())
    }

    pub(super) fn check_capabilities(
        &self,
        requested: State,
        absent: bool,
    ) -> Result<(), TransitionError<B::Error>> {
        let capabilities = self.backend.capabilities();
        if !capabilities.supports(requested.presentation())
            || (absent && !capabilities.launch)
            || (requested == State::Active && !capabilities.passive && !capabilities.hidden)
        {
            return Err(Self::error(Stage::Capabilities, Cause::Unsupported));
        }
        Ok(())
    }
}
