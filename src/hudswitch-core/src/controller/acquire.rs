use super::{Controller, Managed, TargetId};
use crate::*;

impl<B: Backend> Controller<B> {
    pub(super) fn acquire(
        &mut self,
        target: &mut Option<Managed<B>>,
        requested: State,
    ) -> Result<(), TransitionError<B::Error>> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| Self::error(Stage::Acquire, Cause::GenerationExhausted))?;
        let intent = if requested == State::Active {
            VisibleState::Active
        } else {
            VisibleState::Passive
        };
        let acquired = self
            .backend
            .acquire(intent)
            .map_err(|error| Self::error(Stage::Acquire, Cause::Backend(error)))?;
        self.generation = generation;
        *target = Some(Managed {
            target: acquired.target,
            original: acquired.original,
            focus: None,
            state: None,
            id: TargetId(generation),
        });
        Ok(())
    }
}
