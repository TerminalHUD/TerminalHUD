use crate::{Availability, Backend, State, Status, VisibleState};

mod acquire;
mod apply;
mod dispatch;
mod failure;
mod recovery;
mod refresh;
mod shutdown;
mod transition;
mod validate;

/// Controller-local generation for rejecting stale lifecycle notifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetId(u64);

struct Managed<B: Backend> {
    target: B::Target,
    original: B::Original,
    focus: Option<B::Focus>,
    state: Option<State>,
    id: TargetId,
}

pub struct Controller<B: Backend> {
    backend: B,
    managed: Option<Managed<B>>,
    stopped: bool,
    generation: u64,
    creation_state: Option<VisibleState>,
}

impl<B: Backend> Controller<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            managed: None,
            stopped: false,
            generation: 0,
            creation_state: None,
        }
    }

    /// Select the visible state for transitions that create a missing target.
    /// Existing targets and events that do not create remain unchanged.
    pub fn with_creation_state(backend: B, state: VisibleState) -> Self {
        Self {
            creation_state: Some(state),
            ..Self::new(backend)
        }
    }

    pub fn status(&self) -> Status {
        Status {
            availability: if self.stopped {
                Availability::Unmanaged
            } else if self.managed.is_some() {
                Availability::Managed
            } else {
                Availability::Absent
            },
            state: self.managed.as_ref().and_then(|target| target.state),
            target_id: self.managed.as_ref().map(|target| target.id),
        }
    }
}
