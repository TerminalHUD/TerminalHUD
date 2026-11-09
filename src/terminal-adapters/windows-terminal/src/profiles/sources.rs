use super::{
    Catalog, Profile,
    inheritance::{effective, normalize, overlay},
    parse::{document, profiles},
};
use crate::{Error, Guid};
use serde_json::{Map, Value};

/// A discovered fragment in Terminal's load order. Relative media resources must
/// be resolved against the fragment directory by the caller before supplying it.
pub struct ProfileFragment<'a> {
    pub namespace: &'a str,
    pub jsonc: &'a str,
}
/// An actual provider result, including explicit GUID and launch properties.
pub struct GeneratedProfile<'a> {
    pub source: &'a str,
    pub profile: Value,
}
struct Entry {
    guid: Guid,
    values: Map<String, Value>,
}
fn entry(value: &Value) -> Result<Option<Entry>, Error> {
    let values = value
        .as_object()
        .ok_or(Error::Invalid("invalid profile entry"))?;
    let Some(guid) = values.get("guid").and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(Entry {
        guid: Guid::parse(guid)?,
        values: normalize(values.clone()),
    }))
}
fn entries(values: &[Value], unresolved: &mut usize) -> Result<Vec<Entry>, Error> {
    let mut result: Vec<Entry> = Vec::new();
    for value in values {
        if let Some(entry) = entry(value)? {
            if result.iter().any(|other| other.guid == entry.guid) {
                return Err(Error::Collision);
            }
            result.push(entry);
        } else {
            *unresolved += 1;
        }
    }
    Ok(result)
}
impl Catalog {
    /// Pure preparation from an installed runtime's supplied sources. This does
    /// not assert that an already-running Terminal has reloaded any of them.
    pub fn from_sources(
        settings: &str,
        inbox: Option<&str>,
        fragments: &[ProfileFragment<'_>],
        generated: &[GeneratedProfile<'_>],
    ) -> Result<Self, Error> {
        let root = document(settings)?;
        let inbox = inbox.map(document).transpose()?.unwrap_or_default();
        let (defaults, user) = profiles(&root)?;
        let (inbox_defaults, inbox_profiles) = profiles(&inbox)?;
        let mut defaults = normalize(defaults);
        let mut inherited_defaults = normalize(inbox_defaults);
        overlay(&mut inherited_defaults, &defaults);
        defaults = inherited_defaults;
        let mut unresolved_entries = 0;
        let user = entries(&user, &mut unresolved_entries)?;
        let mut parents = entries(&inbox_profiles, &mut unresolved_entries)?;
        let disabled = root.get("disabledProfileSources").and_then(Value::as_array);
        let is_disabled = |source: &str| {
            disabled.is_some_and(|values| values.iter().any(|value| value.as_str() == Some(source)))
        };
        for generated in generated {
            if is_disabled(generated.source) {
                continue;
            }
            let mut generated_entry = entry(&generated.profile)?.ok_or(Error::UnresolvedSource)?;
            generated_entry
                .values
                .insert("source".into(), Value::String(generated.source.into()));
            if parents.iter().any(|p| p.guid == generated_entry.guid) {
                return Err(Error::Collision);
            }
            parents.push(generated_entry);
        }
        let mut updates = Vec::new();
        for fragment in fragments {
            if is_disabled(fragment.namespace) {
                continue;
            }
            let fragment_root = document(fragment.jsonc)?;
            let values = fragment_root
                .get("profiles")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for value in values {
                if let Some(guid) = value.get("updates").and_then(Value::as_str) {
                    let guid = Guid::parse(guid)?;
                    if updates.iter().any(|p: &Entry| p.guid == guid) {
                        return Err(Error::Invalid("ambiguous fragment updates"));
                    }
                    let mut values = normalize(value.as_object().unwrap().clone());
                    values.remove("updates");
                    updates.push(Entry { guid, values });
                } else if let Some(mut added) = entry(&value)? {
                    if parents.iter().any(|p| p.guid == added.guid) {
                        return Err(Error::Collision);
                    }
                    added
                        .values
                        .insert("source".into(), Value::String(fragment.namespace.into()));
                    parents.push(added);
                } else {
                    unresolved_entries += 1;
                }
            }
        }
        let mut ids: Vec<Guid> = user.iter().map(|p| p.guid.clone()).collect();
        for parent in &parents {
            if !ids.contains(&parent.guid) {
                ids.push(parent.guid.clone());
            }
        }
        let mut resolved = Vec::new();
        for guid in ids {
            let user = user.iter().find(|p| p.guid == guid);
            let parent = parents.iter().find(|p| p.guid == guid);
            let mut layers = vec![
                user.map(|p| p.values.clone()).unwrap_or_default(),
                defaults.clone(),
            ];
            if let Some(update) = updates.iter().find(|p| p.guid == guid) {
                layers.push(update.values.clone());
            }
            if let Some(parent) = parent {
                layers.push(parent.values.clone());
            }
            let values = effective(&layers);
            let source = values
                .get("source")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty());
            let source_resolved = source.is_none()
                || parent.is_some_and(|p| p.values.get("source").and_then(Value::as_str) == source);
            let resolved_commandline = source_resolved
                && values
                    .get("commandline")
                    .and_then(Value::as_str)
                    .is_some_and(|s| !s.trim().is_empty());
            let name = values
                .get("name")
                .and_then(Value::as_str)
                .ok_or(Error::Invalid("profile name is missing"))?
                .to_owned();
            resolved.push(Profile {
                guid,
                name,
                effective: values,
                resolved_commandline,
            });
        }
        let default_profile = root
            .get("defaultProfile")
            .or_else(|| inbox.get("defaultProfile"))
            .map(|v| {
                Guid::parse(
                    v.as_str()
                        .ok_or(Error::Invalid("default profile must be an explicit GUID"))?,
                )
            })
            .transpose()?;
        Ok(Self {
            default_profile,
            profiles: resolved,
            unresolved_entries,
            defaults,
        })
    }
}
