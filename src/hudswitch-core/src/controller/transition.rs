use crate::{Event, State, VisibleState};

pub(super) fn requested_state(event: Event, state: Option<State>) -> Option<State> {
    let requested = match (event, state) {
        (Event::InteractionToggle, Some(State::Active)) => State::Passive,
        (Event::InteractionToggle, _) => State::Active,
        (Event::VisibilityToggle, Some(State::Active)) => State::Hidden {
            return_state: VisibleState::Active,
        },
        (Event::VisibilityToggle, Some(State::Passive)) => State::Hidden {
            return_state: VisibleState::Passive,
        },
        (
            Event::VisibilityToggle,
            Some(State::Hidden {
                return_state: VisibleState::Active,
            }),
        ) => State::Active,
        (
            Event::VisibilityToggle,
            Some(State::Hidden {
                return_state: VisibleState::Passive,
            }),
        ) => State::Passive,
        (Event::VisibilityToggle, None) => State::Passive,
        (Event::RequestPassive | Event::FocusLost(_), Some(State::Active)) => State::Passive,
        _ => return None,
    };
    Some(requested)
}
