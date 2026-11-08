use super::{Controller, Managed};
use crate::*;

impl<B: Backend> Controller<B> {
    /// Explicitly retry safe Passive, then Hidden, for an unconfirmed target.
    /// Retains ownership and the original snapshot if recovery remains impossible.
    pub fn recover(&mut self) -> Result<Outcome<B::Error>, TransitionError<B::Error>> {
        let mut target = self.managed.take();
        let mut failures = Vec::new();
        let recovery = self.make_safe(&mut target, true, &mut failures);
        self.managed = target;
        match recovery {
            Recovery::Unconfirmed => Err(TransitionError {
                failure: AttemptFailure {
                    stage: Stage::Observe,
                    cause: Cause::RecoveryRequired,
                },
                recovery,
                recovery_failures: failures,
            }),
            _ => Ok(Outcome {
                status: self.status(),
                focus_restore_error: None,
            }),
        }
    }

    pub(super) fn make_safe(
        &mut self,
        target: &mut Option<Managed<B>>,
        preserve_foreground: bool,
        failures: &mut Vec<AttemptFailure<B::Error>>,
    ) -> Recovery {
        let Some(current) = target.as_mut() else {
            return Recovery::Absent;
        };
        current.state = None;
        match self.backend.is_valid(&current.target) {
            Ok(false) => {
                *target = None;
                return Recovery::Absent;
            }
            Err(error) => {
                failures.push(AttemptFailure {
                    stage: Stage::Validate,
                    cause: Cause::Backend(error),
                });
                return Recovery::Unconfirmed;
            }
            Ok(true) => {}
        }
        let capabilities = self.backend.capabilities();
        for (state, recovery) in [
            (State::Passive, Recovery::Passive),
            (
                State::Hidden {
                    return_state: VisibleState::Passive,
                },
                Recovery::Hidden,
            ),
        ] {
            if capabilities.supports(state.presentation()) {
                match self.apply(current, state, preserve_foreground) {
                    Ok(warning) => {
                        if let Some(error) = warning {
                            failures.push(AttemptFailure {
                                stage: Stage::RestoreFocus,
                                cause: Cause::Backend(error),
                            });
                        }
                        return recovery;
                    }
                    Err(error) => failures.push(error),
                }
            }
        }
        Recovery::Unconfirmed
    }
}
