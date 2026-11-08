use super::{Controller, Managed};
use crate::*;

impl<B: Backend> Controller<B> {
    pub(super) fn apply(
        &mut self,
        target: &mut Managed<B>,
        state: State,
        preserve_foreground: bool,
    ) -> Result<Option<B::Error>, AttemptFailure<B::Error>> {
        target.state = None;
        let presentation = state.presentation();
        self.backend
            .present(&target.target, presentation)
            .map_err(|error| AttemptFailure {
                stage: Stage::Present,
                cause: Cause::Backend(error),
            })?;
        let mut warning = None;
        if state == State::Active {
            self.backend
                .activate(&target.target)
                .map_err(|error| AttemptFailure {
                    stage: Stage::Activate,
                    cause: Cause::Backend(error),
                })?;
        } else {
            let focus = target.focus.take();
            if !preserve_foreground && let Some(focus) = focus {
                warning = self.backend.restore_focus(&focus).err();
            }
            self.backend
                .release_focus_if_owned(&target.target)
                .map_err(|error| AttemptFailure {
                    stage: Stage::ReleaseFocus,
                    cause: Cause::Backend(error),
                })?;
        }
        let observation = self
            .backend
            .observe(&target.target)
            .map_err(|error| AttemptFailure {
                stage: Stage::Observe,
                cause: Cause::Backend(error),
            })?;
        if !observation.confirms(presentation) {
            return Err(AttemptFailure {
                stage: Stage::Observe,
                cause: Cause::InvariantMismatch(observation),
            });
        }
        target.state = Some(state);
        Ok(warning)
    }
}
