use super::{object::Object, policy, wire::*};
use terminalhud_core::config::{
    Candidate, Configuration, ProfileId, ProfileSelection, ValidationError,
};

/// Parse and validate a complete candidate without modifying existing state.
pub fn decode(json: &str) -> Result<Configuration, ValidationError> {
    let mut deserializer = serde_json::Deserializer::from_str(json);
    let Object(document): Object<Document> = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|error| ValidationError::new("JSON", error.to_string()))?;
    deserializer
        .end()
        .map_err(|error| ValidationError::new("JSON", error.to_string()))?;
    policy::validate(&document)?;
    let profile = match document.terminal.default_hud_profile.as_str() {
        "auto" => ProfileSelection::Auto,
        id => ProfileSelection::Id(ProfileId::new(id)?),
    };
    Configuration::validate(Candidate {
        toggle_interaction: document.hotkeys.toggle_interaction,
        toggle_visibility: document.hotkeys.toggle_visibility,
        start_with_windows: document.startup.start_with_windows,
        active_background_opacity: document.terminal.active_background_opacity,
        default_hud_profile: profile,
    })
}

/// Commit only after both syntax and complete semantic validation succeed.
pub fn replace_from_json(current: &mut Configuration, json: &str) -> Result<(), ValidationError> {
    *current = decode(json)?;
    Ok(())
}

pub fn encode(config: &Configuration) -> Result<String, serde_json::Error> {
    let profile = match config.default_hud_profile() {
        ProfileSelection::Auto => "auto".into(),
        ProfileSelection::Id(id) => id.as_str().into(),
    };
    serde_json::to_string_pretty(&Document {
        version: 1,
        hotkeys: Hotkeys {
            toggle_interaction: config.toggle_interaction().to_string(),
            toggle_visibility: config.toggle_visibility().to_string(),
        },
        startup: Startup {
            start_with_windows: config.start_with_windows(),
        },
        terminal: Terminal {
            active_background_opacity: i64::from(config.active_background_opacity()),
            default_hud_profile: profile,
            ..Terminal::default()
        },
        display: Display::default(),
        lifecycle: Lifecycle::default(),
    })
}
