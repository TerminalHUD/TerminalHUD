/// A visible state that can be remembered while the target is hidden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibleState {
    Active,
    Passive,
}

/// Logical state, separate from target availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Active,
    Passive,
    Hidden { return_state: VisibleState },
}

/// Requested backend presentation; hidden-state memory stays in the core.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presentation {
    Active,
    Passive,
    Hidden,
}

impl State {
    pub(crate) fn presentation(self) -> Presentation {
        match self {
            Self::Active => Presentation::Active,
            Self::Passive => Presentation::Passive,
            Self::Hidden { .. } => Presentation::Hidden,
        }
    }
}
