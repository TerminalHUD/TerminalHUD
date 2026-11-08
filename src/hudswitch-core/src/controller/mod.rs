use crate::{Availability, Backend, State, Status};

mod acquire;
mod apply;
mod dispatch;
mod failure;
mod recovery;
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
}

impl<B: Backend> Controller<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            managed: None,
            stopped: false,
            generation: 0,
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
