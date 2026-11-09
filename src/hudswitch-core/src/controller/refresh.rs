use super::Controller;
use crate::*;
impl<B: Backend> Controller<B> {
    /// Reconcile owned native facts without acquiring a missing target. Healthy
    /// observations preserve focus history and Hidden return state unchanged.
    pub fn refresh(&mut self) -> Result<Outcome<B::Error>, TransitionError<B::Error>> {
        let mut target = self.managed.take();
        let result = self.refresh_target(&mut target);
        self.managed = target;
        if let Some(id) = result? {
            return self.dispatch(Event::FocusLost(id));
        }
        Ok(Outcome {
            status: self.status(),
            focus_restore_error: None,
        })
    }
    fn refresh_target(
        &mut self,
        target: &mut Option<super::Managed<B>>,
    ) -> Result<Option<TargetId>, TransitionError<B::Error>> {
        self.validate_target(target)?;
        let Some(current) = target.as_ref() else {
            return Ok(None);
        };
        let observation = match self.backend.observe(&current.target) {
            Ok(observation) => observation,
            Err(error) => {
                return Err(self.failed(target, Stage::Observe, Cause::Backend(error), true));
            }
        };
        if current.state == Some(State::Active) && !observation.focused {
            return Ok(Some(current.id));
        }
        if !current
            .state
            .is_some_and(|state| observation.confirms(state.presentation()))
        {
            return Err(self.failed(
                target,
                Stage::Observe,
                Cause::InvariantMismatch(observation),
                true,
            ));
        }
        Ok(None)
    }
}
