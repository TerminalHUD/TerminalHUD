use super::{Controller, Managed};
use crate::*;

impl<B: Backend> Controller<B> {
    pub(super) fn shutdown(
        &mut self,
        target: &mut Option<Managed<B>>,
    ) -> Result<Option<B::Error>, TransitionError<B::Error>> {
        if let Some(current) = target.as_mut() {
            current.state = None;
            if let Err(error) = self
                .backend
                .restore_original(&current.target, &current.original)
            {
                return Err(self.failed(
                    target,
                    Stage::RestoreOriginal,
                    Cause::Backend(error),
                    false,
                ));
            }
        }
        *target = None;
        self.stopped = true;
        Ok(None)
    }
}
