use crate::{Observation, Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Capabilities,
    Acquire,
    Validate,
    CaptureFocus,
    Present,
    Activate,
    RestoreFocus,
    ReleaseFocus,
    Observe,
    RestoreOriginal,
}

#[derive(Debug)]
pub enum Cause<E> {
    Backend(E),
    Unsupported,
    InvariantMismatch(Observation),
    RecoveryRequired,
    Stopped,
    GenerationExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recovery {
    NotNeeded,
    Passive,
    Hidden,
    Absent,
    Unconfirmed,
}

#[derive(Debug)]
pub struct AttemptFailure<E> {
    pub stage: Stage,
    pub cause: Cause<E>,
}

#[derive(Debug)]
pub struct TransitionError<E> {
    pub failure: AttemptFailure<E>,
    pub recovery: Recovery,
    pub recovery_failures: Vec<AttemptFailure<E>>,
}

/// Failure to restore a previous foreground is a warning after safe completion.
#[derive(Debug)]
pub struct Outcome<E> {
    pub status: Status,
    pub focus_restore_error: Option<E>,
}
