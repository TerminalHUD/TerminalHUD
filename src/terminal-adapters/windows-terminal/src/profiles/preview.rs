use super::Catalog;
use crate::{Error, Guid};
use serde_json::{Map, Value, json};
impl Catalog {
    /// Preview a new HUD variant. Unsupported dynamic launch sources and elevated
    /// profiles require deliberate resolution rather than guessed commands/UAC.
    pub fn preview(
        &self,
        source: &Guid,
        new_guid: Guid,
        active_opacity: u8,
    ) -> Result<FragmentPreview, Error> {
        if active_opacity > 100 {
            return Err(Error::Invalid("opacity must be 0..100"));
        }
        if self.profiles.iter().any(|profile| profile.guid == new_guid) {
            return Err(Error::Collision);
        }
        let source = self
            .profiles
            .iter()
            .find(|profile| &profile.guid == source)
            .ok_or(Error::MissingProfile)?;
        if !source.has_resolved_commandline() {
            return Err(Error::UnresolvedSource);
        }
        if source.elevated() {
            return Err(Error::ElevatedProfile);
        }
        let mut profile = source.effective.clone();
        for key in ["guid", "name", "source", "updates", "hidden"] {
            profile.remove(key);
        }
        profile.insert("guid".into(), json!(new_guid.as_str()));
        profile.insert("name".into(), json!(format!("HUD — {}", source.name)));
        profile.insert("hidden".into(), json!(false));
        profile.insert("opacity".into(), json!(active_opacity));
        profile.insert("useAcrylic".into(), json!(false));
        let mut unfocused = match profile.remove("unfocusedAppearance") {
            Some(Value::Object(value)) => value,
            None | Some(Value::Null) => Map::new(),
            _ => {
                return Err(Error::Invalid(
                    "unfocusedAppearance must be an object or null",
                ));
            }
        };
        unfocused.insert("opacity".into(), json!(0));
        unfocused.insert("useAcrylic".into(), json!(false));
        unfocused.insert("backgroundImageOpacity".into(), json!(0));
        profile.insert("unfocusedAppearance".into(), Value::Object(unfocused));
        FragmentPreview::from_profile(profile)
    }
}

#[derive(Clone)]
pub struct FragmentPreview {
    value: Value,
}
impl FragmentPreview {
    fn from_profile(profile: Map<String, Value>) -> Result<Self, Error> {
        if profile.contains_key("updates") || profile.contains_key("source") {
            return Err(Error::Invalid("HUD fragments create independent profiles"));
        }
        Ok(Self {
            value: json!({ "profiles": [profile] }),
        })
    }
    pub fn json(&self) -> &Value {
        &self.value
    }
    pub fn utf8(&self) -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&self.value).expect("JSON values serialize")
        )
    }
}
