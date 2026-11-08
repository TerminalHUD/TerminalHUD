use crate::{Capabilities, Observation, Presentation, VisibleState};

/// Acquired target and its original state, captured before controller mutation.
/// Dropping either token must not terminate or close the target application.
pub struct Acquired<Target, Original> {
    pub target: Target,
    pub original: Original,
}

/// Platform contract. Methods must validate identity before touching a target.
/// No method may change terminal appearance, opacity, or unrelated windows.
pub trait Backend {
    type Target;
    type Focus;
    type Original;
    type Error;

    fn capabilities(&self) -> Capabilities;

    /// Launch/acquire only the intended target and capture original owned state.
    /// Passive intent must not steal focus, even briefly. On error, the backend
    /// retains responsibility for leaving any created target ordinary and usable.
    fn acquire(
        &mut self,
        intent: VisibleState,
    ) -> Result<Acquired<Self::Target, Self::Original>, Self::Error>;

    fn is_valid(&mut self, target: &Self::Target) -> Result<bool, Self::Error>;
    fn foreground(&mut self) -> Result<Option<Self::Focus>, Self::Error>;

    /// Apply visibility, topmost, and input style changes, preserving all bits
    /// outside the backend's ownership. This does not activate another window.
    fn present(
        &mut self,
        target: &Self::Target,
        presentation: Presentation,
    ) -> Result<(), Self::Error>;

    fn activate(&mut self, target: &Self::Target) -> Result<(), Self::Error>;

    /// Attempt the saved window once, only if it is still valid and eligible.
    fn restore_focus(&mut self, focus: &Self::Focus) -> Result<(), Self::Error>;

    /// Relinquish target focus only if it still owns focus; preserve a foreground
    /// window the user selected meanwhile. Let the OS choose when no saved
    /// foreground can be restored; do not force or retry another application.
    fn release_focus_if_owned(&mut self, target: &Self::Target) -> Result<(), Self::Error>;

    fn observe(&mut self, target: &Self::Target) -> Result<Observation, Self::Error>;

    /// Restore only changes owned by this backend from the original snapshot.
    /// Leave the target open and usable. Returning success certifies completion.
    fn restore_original(
        &mut self,
        target: &Self::Target,
        original: &Self::Original,
    ) -> Result<(), Self::Error>;
}
