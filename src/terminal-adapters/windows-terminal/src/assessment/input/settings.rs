use super::super::{
    properties,
    types::{AssessmentFinding, Provenance, RefusalReason, finding},
};
use super::Layer;
use super::entries::{parse_guid, reject_updates};
use crate::Guid;
use serde_json::{Map, Value};

pub(super) fn default_profile(
    settings: &Map<String, Value>,
) -> Result<Option<Guid>, AssessmentFinding> {
    settings
        .get("defaultProfile")
        .map(|v| parse_guid(v, "settings.defaultProfile"))
        .transpose()
}

pub(super) fn user_profiles(
    settings: &Map<String, Value>,
) -> Result<(Layer, &[Value]), AssessmentFinding> {
    let mut defaults = Map::new();
    let list = match settings.get("profiles") {
        None => &[][..],
        Some(Value::Array(list)) => list.as_slice(),
        Some(Value::Object(container)) => {
            if let Some(value) = container.get("defaults") {
                defaults = value.as_object().cloned().ok_or_else(|| {
                    finding(
                        RefusalReason::InvalidContainer,
                        "settings.profiles.defaults",
                    )
                })?;
            }
            match container.get("list") {
                None => &[][..],
                Some(Value::Array(list)) => list.as_slice(),
                _ => {
                    return Err(finding(
                        RefusalReason::InvalidContainer,
                        "settings.profiles.list",
                    ));
                }
            }
        }
        _ => {
            return Err(finding(
                RefusalReason::InvalidContainer,
                "settings.profiles",
            ));
        }
    };
    // CascadiaSettingsSerialization.cpp _parse clears these before inheritance.
    for key in ["guid", "name", "source", "commandline"] {
        defaults.remove(key);
    }
    let location = "settings.profiles.defaults";
    reject_updates(&defaults, location)?;
    properties::validate(&defaults, location)?;
    Ok((
        Layer {
            values: defaults,
            provenance: Provenance::UserDefaults,
            location: location.into(),
        },
        list,
    ))
}

pub(super) fn disabled_sources(
    settings: &Map<String, Value>,
) -> Result<Vec<String>, AssessmentFinding> {
    match settings.get("disabledProfileSources") {
        None => Ok(Vec::new()),
        Some(Value::Array(values)) if values.iter().all(Value::is_string) => Ok(values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect()),
        _ => Err(finding(
            RefusalReason::InvalidField,
            "settings.disabledProfileSources",
        )),
    }
}
