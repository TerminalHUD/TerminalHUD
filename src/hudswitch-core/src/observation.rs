use crate::Presentation;

/// Observable backend facts used to confirm the requested invariant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub visible: bool,
    pub topmost: bool,
    pub click_through: bool,
    pub focused: bool,
}

impl Observation {
    pub(crate) fn confirms(self, presentation: Presentation) -> bool {
        match presentation {
            Presentation::Active => {
                self.visible && self.topmost && !self.click_through && self.focused
            }
            Presentation::Passive => {
                self.visible && self.topmost && self.click_through && !self.focused
            }
            Presentation::Hidden => !self.visible && !self.focused,
        }
    }
}
