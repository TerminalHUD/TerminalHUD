use super::{Controller, Managed};
use crate::{AttemptFailure, Backend, Cause, Recovery, Stage, TransitionError};

impl<B: Backend> Controller<B> {
    pub(super) fn failed(
        &mut self,
        target: &mut Option<Managed<B>>,
        stage: Stage,
        cause: Cause<B::Error>,
        preserve_foreground: bool,
    ) -> TransitionError<B::Error> {
        let mut recovery_failures = Vec::new();
        let recovery = self.make_safe(target, preserve_foreground, &mut recovery_failures);
        TransitionError {
            failure: AttemptFailure { stage, cause },
            recovery,
            recovery_failures,
        }
    }

    pub(super) fn error(stage: Stage, cause: Cause<B::Error>) -> TransitionError<B::Error> {
        TransitionError {
            failure: AttemptFailure { stage, cause },
            recovery: Recovery::NotNeeded,
            recovery_failures: Vec::new(),
        }
    }
}
