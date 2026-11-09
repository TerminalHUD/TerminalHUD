use super::{Catalog, FragmentPreview};
use crate::{Error, Guid};
use serde_json::Value;
use std::collections::BTreeMap;

/// Launch preparation, not proof that a running Terminal has loaded the profile.
/// `owned_profile` belongs at user-list precedence when defaults conflict with a
/// fragment. Writing that entry requires separately approved application policy.
pub struct PreparedProfile {
    pub source_guid: Guid,
    pub hud_guid: Guid,
    pub fragment: FragmentPreview,
    pub owned_profile: Value,
    pub fragment_compatible: bool,
    pub commandline: String,
    pub starting_directory: Option<String>,
    pub environment: BTreeMap<String, String>,
    /// Effective compatibility.reloadEnvironmentVariables; Terminal defaults to true.
    pub reload_environment: bool,
}
impl Catalog {
    pub fn prepare(
        &self,
        selected: Option<&Guid>,
        hud_guid: Guid,
        active_opacity: u8,
    ) -> Result<PreparedProfile, Error> {
        let selected = selected
            .or(self.default_profile.as_ref())
            .ok_or(Error::MissingProfile)?;
        let source = self
            .profiles
            .iter()
            .find(|p| &p.guid == selected)
            .ok_or(Error::MissingProfile)?;
        if source.hidden() {
            return Err(Error::Invalid("selected profile is hidden"));
        }
        validate_connection(&source.effective)?;
        let fragment = self.preview(selected, hud_guid.clone(), active_opacity)?;
        let owned_profile = fragment.json()["profiles"][0].clone();
        let values = owned_profile
            .as_object()
            .expect("preview profile is an object");
        let loaded_fragment = super::inheritance::effective(&[
            Default::default(),
            self.default_layers[0].clone(),
            self.default_layers[1].clone(),
            values.clone(),
        ]);
        let fragment_compatible = values
            .iter()
            .all(|(key, value)| loaded_fragment.get(key) == Some(value));
        let commandline = source
            .effective
            .get("commandline")
            .and_then(Value::as_str)
            .ok_or(Error::UnresolvedSource)?
            .to_owned();
        let starting_directory = source
            .effective
            .get("startingDirectory")
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or(Error::Invalid("invalid starting directory"))
            })
            .transpose()?;
        let mut environment = BTreeMap::new();
        if let Some(value) = source.effective.get("environment") {
            for (key, value) in value
                .as_object()
                .ok_or(Error::Invalid("invalid profile environment"))?
            {
                environment.insert(
                    key.clone(),
                    value
                        .as_str()
                        .ok_or(Error::Invalid("invalid environment value"))?
                        .to_owned(),
                );
            }
        }
        let reload_environment = source
            .effective
            .get("compatibility.reloadEnvironmentVariables")
            .map(|value| {
                value
                    .as_bool()
                    .ok_or(Error::Invalid("invalid environment reload setting"))
            })
            .transpose()?
            .unwrap_or(true);
        Ok(PreparedProfile {
            source_guid: selected.clone(),
            hud_guid,
            fragment,
            owned_profile,
            fragment_compatible,
            commandline,
            starting_directory,
            environment,
            reload_environment,
        })
    }
}

impl Catalog {
    /// An unknown -p GUID may fall back before the bridge can reject its identity.
    /// Check both native defaults and configured fallback without requiring the
    /// replacement bridge to execute any ordinary shell command.
    pub fn validate_fallback(&self) -> Result<(), Error> {
        validate_connection(&self.defaults)?;
        if let Ok(default) = self.default() {
            validate_connection(&default.effective)?;
        }
        Ok(())
    }
}
fn validate_connection(values: &serde_json::Map<String, Value>) -> Result<(), Error> {
    if let Some(elevate) = values.get("elevate") {
        match elevate.as_bool() {
            Some(false) => (),
            Some(true) => return Err(Error::ElevatedProfile),
            None => return Err(Error::Invalid("invalid elevation setting")),
        }
    }
    if let Some(connection) = values.get("connectionType")
        && connection
            .as_str()
            .and_then(|s| Guid::parse(s).ok())
            .is_none_or(|g| g.as_str() != "{00000000-0000-0000-0000-000000000000}")
    {
        return Err(Error::Invalid("profile requires an unsupported connection"));
    }
    Ok(())
}
