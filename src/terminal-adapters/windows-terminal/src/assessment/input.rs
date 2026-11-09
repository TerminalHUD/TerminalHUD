use super::types::{AssessmentFinding, Provenance, RefusalReason, SuppliedSnapshot, finding};
use super::{document, properties};
use crate::Guid;
use serde_json::{Map, Value};
use std::collections::HashSet;

pub(super) struct Layer {
    pub values: Map<String, Value>,
    pub provenance: Provenance,
    pub location: String,
}
pub(super) struct Entry {
    pub guid: Guid,
    pub layer: Layer,
}
pub(super) struct ParsedSnapshot {
    pub defaults: Layer,
    pub user: Vec<Entry>,
    pub fragments: Vec<Entry>,
    pub disabled_sources: Vec<String>,
    pub default_profile: Option<Guid>,
}

impl ParsedSnapshot {
    pub fn parse(snapshot: SuppliedSnapshot<'_>) -> Result<Self, AssessmentFinding> {
        let settings = document::parse(snapshot.settings_jsonc, "settings")?;
        let default_profile = settings
            .get("defaultProfile")
            .map(|v| parse_guid(v, "settings.defaultProfile"))
            .transpose()?;
        let disabled_sources = disabled_sources(&settings)?;
        let (defaults, list) = user_profiles(&settings)?;
        let user = entries(list, Provenance::UserProfile, "settings.profiles", false)?;
        let mut fragments = Vec::new();
        let mut labels = HashSet::new();
        for fragment in snapshot.terminalhud_fragments {
            if fragment.namespace != "TerminalHUD" {
                return Err(finding(
                    RefusalReason::UnsupportedNamespace,
                    fragment.namespace,
                ));
            }
            if fragment.document_id.is_empty() || !labels.insert(fragment.document_id) {
                return Err(finding(
                    RefusalReason::DuplicateFragmentInput,
                    fragment.document_id,
                ));
            }
            let label = format!("fragment:{}.{}", fragment.namespace, fragment.document_id);
            let root = document::parse(fragment.jsonc, &label)?;
            let list = root
                .get("profiles")
                .and_then(Value::as_array)
                .ok_or_else(|| finding(RefusalReason::InvalidContainer, &label))?;
            let provenance = Provenance::Fragment {
                namespace: fragment.namespace.into(),
                document_id: fragment.document_id.into(),
            };
            for entry in entries(list, provenance, &label, true)? {
                if fragments.iter().any(|e: &Entry| e.guid == entry.guid) {
                    return Err(finding(
                        RefusalReason::ConflictingFragmentGuid,
                        entry.layer.location,
                    ));
                }
                fragments.push(entry);
            }
        }
        Ok(Self {
            defaults,
            user,
            fragments,
            disabled_sources,
            default_profile,
        })
    }
}

fn user_profiles(settings: &Map<String, Value>) -> Result<(Layer, &[Value]), AssessmentFinding> {
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

fn entries(
    list: &[Value],
    provenance: Provenance,
    label: &str,
    fragment: bool,
) -> Result<Vec<Entry>, AssessmentFinding> {
    let mut entries = Vec::new();
    for (index, value) in list.iter().enumerate() {
        let location = format!("{label}[{index}]");
        let values = value
            .as_object()
            .cloned()
            .ok_or_else(|| finding(RefusalReason::InvalidContainer, &location))?;
        reject_updates(&values, &location)?;
        if fragment && values.contains_key("source") {
            return Err(finding(RefusalReason::UnresolvedSource, location));
        }
        let guid = values
            .get("guid")
            .ok_or_else(|| finding(RefusalReason::MissingGuid, &location))?;
        let guid = parse_guid(guid, &format!("{location}.guid"))?;
        if entries.iter().any(|e: &Entry| e.guid == guid) {
            return Err(finding(RefusalReason::DuplicateGuid, location));
        }
        properties::validate(&values, &location)?;
        entries.push(Entry {
            guid,
            layer: Layer {
                values,
                provenance: provenance.clone(),
                location,
            },
        });
    }
    Ok(entries)
}

fn parse_guid(value: &Value, label: &str) -> Result<Guid, AssessmentFinding> {
    value
        .as_str()
        .and_then(|s| Guid::parse(s).ok())
        .filter(|guid| guid.bare() != "00000000-0000-0000-0000-000000000000")
        .ok_or_else(|| finding(RefusalReason::InvalidField, label))
}
fn reject_updates(values: &Map<String, Value>, label: &str) -> Result<(), AssessmentFinding> {
    if values.contains_key("updates") {
        Err(finding(RefusalReason::UnsupportedUpdate, label))
    } else {
        Ok(())
    }
}
fn disabled_sources(settings: &Map<String, Value>) -> Result<Vec<String>, AssessmentFinding> {
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
