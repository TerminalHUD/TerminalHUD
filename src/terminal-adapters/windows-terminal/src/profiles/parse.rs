use super::{Catalog, Profile};
use crate::{Error, Guid};
use serde_json::{Map, Value};
fn merge(base: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (key, value) in overlay {
        if let (Some(Value::Object(existing)), Value::Object(update)) = (base.get_mut(key), value) {
            merge(existing, update);
        } else {
            base.insert(key.clone(), value.clone());
        }
    }
}

impl Catalog {
    /// Parse supplied JSONC bytes without writing settings, resolving sources,
    /// evaluating commands, or starting applications.
    pub fn parse(text: &str) -> Result<Self, Error> {
        let settings: Value = jsonc_parser::parse_to_serde_value(
            text.trim_start_matches('\u{feff}'),
            &Default::default(),
        )
        .map_err(|error| Error::Settings(error.to_string()))?;
        let root = settings
            .as_object()
            .ok_or(Error::Invalid("settings must be an object"))?;
        let default_profile = root
            .get("defaultProfile")
            .and_then(Value::as_str)
            .and_then(|value| Guid::parse(value).ok());
        let container = root
            .get("profiles")
            .ok_or(Error::Invalid("profiles are missing"))?;
        let mut defaults = container
            .get("defaults")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        // Terminal's pinned loader clears these invalid profile-default keys.
        for key in ["guid", "name", "source", "commandline"] {
            defaults.remove(key);
        }
        let list = container
            .as_array()
            .or_else(|| container.get("list").and_then(Value::as_array))
            .ok_or(Error::Invalid("profile list is missing"))?;
        let mut profiles: Vec<Profile> = Vec::new();
        let mut unresolved_entries = 0;
        for entry in list {
            let Some(profile) = entry.as_object() else {
                return Err(Error::Invalid("profile must be an object"));
            };
            let Some(guid) = profile
                .get("guid")
                .and_then(Value::as_str)
                .and_then(|value| Guid::parse(value).ok())
            else {
                unresolved_entries += 1;
                continue;
            };
            if profiles.iter().any(|existing| existing.guid == guid) {
                return Err(Error::Invalid("duplicate profile GUID"));
            }
            let mut effective = defaults.clone();
            merge(&mut effective, profile);
            let name = profile
                .get("name")
                .and_then(Value::as_str)
                .ok_or(Error::Invalid("profile name is missing"))?
                .to_owned();
            let explicit_command = profile
                .get("commandline")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty());
            profiles.push(Profile {
                guid,
                name,
                effective,
                resolved_commandline: explicit_command,
            });
        }
        Ok(Self {
            default_profile,
            profiles,
            unresolved_entries,
        })
    }

    pub fn default(&self) -> Result<&Profile, Error> {
        let guid = self.default_profile.as_ref().ok_or(Error::MissingProfile)?;
        self.profiles
            .iter()
            .find(|profile| &profile.guid == guid)
            .ok_or(Error::MissingProfile)
    }
}
