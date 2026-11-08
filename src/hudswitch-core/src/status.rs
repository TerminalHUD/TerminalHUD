use crate::{State, TargetId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Absent,
    Managed,
    Unmanaged,
}

/// Confirmed logical state. `Managed` with no state means recovery is required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub availability: Availability,
    pub state: Option<State>,
    pub target_id: Option<TargetId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    InteractionToggle,
    VisibilityToggle,
    RequestPassive,
    FocusLost(TargetId),
    TargetClosed(TargetId),
    Quit,
}
