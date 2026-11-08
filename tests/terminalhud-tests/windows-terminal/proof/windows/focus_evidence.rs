//! Pure interpretation of a diagnostic observation, independent of Win32 calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Evidence {
    Preserved,
    TargetActivated { observation_invalid: bool },
    ObservationInvalid,
}
pub(crate) fn classify(
    sink: isize,
    target: isize,
    final_window: isize,
    events: &[isize],
) -> Evidence {
    let target_seen = final_window == target || events.contains(&target);
    let observation_invalid = (final_window != sink && final_window != target)
        || events
            .iter()
            .any(|window| *window != sink && *window != target);
    if target_seen {
        Evidence::TargetActivated {
            observation_invalid,
        }
    } else if observation_invalid {
        Evidence::ObservationInvalid
    } else {
        Evidence::Preserved
    }
}
