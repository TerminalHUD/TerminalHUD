use super::{ProfileSelection, Shortcut, ValidationError};

/// Untrusted portable input. Validate the entire candidate before replacing state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub toggle_interaction: String,
    pub toggle_visibility: String,
    pub start_with_windows: bool,
    pub active_background_opacity: i64,
    pub default_hud_profile: ProfileSelection,
}

impl Default for Candidate {
    fn default() -> Self {
        Self {
            toggle_interaction: "Win+Backtick".into(),
            toggle_visibility: "Win+Shift+Backtick".into(),
            start_with_windows: true,
            active_background_opacity: 35,
            default_hud_profile: ProfileSelection::Auto,
        }
    }
}

/// Only validated preferences can be observed through this interface.
/// Creation, presentation and lifecycle policy are fixed outside preferences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    toggle_interaction: Shortcut,
    toggle_visibility: Shortcut,
    start_with_windows: bool,
    active_background_opacity: u8,
    default_hud_profile: ProfileSelection,
}

impl Configuration {
    pub fn validate(candidate: Candidate) -> Result<Self, ValidationError> {
        let toggle_interaction = Shortcut::parse(&candidate.toggle_interaction)
            .map_err(|message| ValidationError::new("hotkeys.toggleInteraction", message))?;
        let toggle_visibility = Shortcut::parse(&candidate.toggle_visibility)
            .map_err(|message| ValidationError::new("hotkeys.toggleVisibility", message))?;
        if toggle_interaction == toggle_visibility {
            return Err(ValidationError::new(
                "hotkeys.toggleVisibility",
                "shortcuts must be distinct",
            ));
        }
        if !(0..=100).contains(&candidate.active_background_opacity) {
            return Err(ValidationError::new(
                "terminal.activeBackgroundOpacity",
                "expected integer 0..=100",
            ));
        }
        Ok(Self {
            toggle_interaction,
            toggle_visibility,
            start_with_windows: candidate.start_with_windows,
            active_background_opacity: candidate.active_background_opacity as u8,
            default_hud_profile: candidate.default_hud_profile,
        })
    }

    pub fn replace(&mut self, candidate: Candidate) -> Result<(), ValidationError> {
        *self = Self::validate(candidate)?;
        Ok(())
    }

    pub fn toggle_interaction(&self) -> &Shortcut {
        &self.toggle_interaction
    }
    pub fn toggle_visibility(&self) -> &Shortcut {
        &self.toggle_visibility
    }
    pub fn start_with_windows(&self) -> bool {
        self.start_with_windows
    }
    pub fn active_background_opacity(&self) -> u8 {
        self.active_background_opacity
    }
    pub fn default_hud_profile(&self) -> &ProfileSelection {
        &self.default_hud_profile
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::validate(Candidate::default()).expect("valid built-in defaults")
    }
}
