use crate::Presentation;

/// Complete operations the backend can support on this platform/session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    pub launch: bool,
    pub active: bool,
    pub passive: bool,
    pub hidden: bool,
}

impl Capabilities {
    pub const ALL: Self = Self {
        launch: true,
        active: true,
        passive: true,
        hidden: true,
    };

    pub(crate) fn supports(self, presentation: Presentation) -> bool {
        match presentation {
            Presentation::Active => self.active,
            Presentation::Passive => self.passive,
            Presentation::Hidden => self.hidden,
        }
    }
}
