//! Proposed owned-profile edits. Pure bytes only: never opens or writes a file.
use crate::{Error, Guid};
use jsonc_parser::common::Ranged;
use serde_json::Value;
use std::{collections::HashSet, ops::Range};

#[derive(Clone, Debug)]
pub struct OwnedProfileEntry {
    pub guid: Guid,
    pub block: String,
}
impl OwnedProfileEntry {
    pub fn to_json(&self) -> Value {
        serde_json::json!({"guid":self.guid.as_str(), "block":self.block})
    }
    pub fn from_json(value: &Value) -> Result<Self, Error> {
        let guid = Guid::parse(
            value
                .get("guid")
                .and_then(Value::as_str)
                .ok_or(Error::Invalid("invalid ownership GUID"))?,
        )?;
        let block = value
            .get("block")
            .and_then(Value::as_str)
            .ok_or(Error::Invalid("invalid ownership block"))?
            .to_owned();
        Ok(Self { guid, block })
    }
}
pub struct ProfilePatch {
    pub original: String,
    pub updated: String,
    pub ownership: OwnedProfileEntry,
}
/// Prepare an insertion/replacement without changing any ordinary settings byte.
/// Application must compare original bytes under its transaction before writing.
pub fn plan_profile_patch(
    settings: &str,
    profile: &Value,
    previous: Option<&OwnedProfileEntry>,
) -> Result<ProfilePatch, Error> {
    let guid = Guid::parse(
        profile
            .get("guid")
            .and_then(Value::as_str)
            .ok_or(Error::Invalid("owned profile requires GUID"))?,
    )?;
    if profile.get("source").is_some() || profile.get("updates").is_some() {
        return Err(Error::Invalid("owned profile must be independent"));
    }
    if previous.is_some_and(|owned| owned.guid != guid) {
        return Err(Error::Invalid("owned profile GUID cannot change"));
    }
    let original = settings.to_owned();
    let mut updated = match previous {
        Some(owned) => remove_owned_profile(settings, owned)?,
        None => original.clone(),
    };
    let range = list_range(&updated)?;
    let root = super::parse::document(&updated)?;
    let (_, profiles) = super::parse::profiles(&root)?;
    if profiles.iter().any(|p| {
        p.get("guid")
            .and_then(Value::as_str)
            .and_then(|s| Guid::parse(s).ok())
            .as_ref()
            == Some(&guid)
    }) {
        return Err(Error::Collision);
    }
    let block =
        owned_block(&guid, profile).map_err(|_| Error::Invalid("cannot serialize profile"))?;
    updated.insert_str(range.start + 1, &block);
    list_range(&updated)?;
    Ok(ProfilePatch {
        original,
        updated,
        ownership: OwnedProfileEntry { guid, block },
    })
}
/// Refuse modified/moved/duplicated owned material; retain unrelated concurrent edits.
pub fn remove_owned_profile(settings: &str, owned: &OwnedProfileEntry) -> Result<String, Error> {
    let (range, entries) = list_layout(settings)?;
    if owned.block.is_empty() {
        return Err(Error::Invalid("empty ownership receipt"));
    }
    let mut matches = settings.match_indices(&owned.block);
    let (start, _) = matches
        .next()
        .ok_or(Error::Invalid("owned profile was modified or removed"))?;
    let end = start + owned.block.len();
    if matches.next().is_some() || start <= range.start || end >= range.end {
        return Err(Error::Invalid("ambiguous ownership receipt"));
    }
    // Receipt must describe exactly one independent profile with the expected GUID.
    let receipt = format!("{{\"profiles\":[{}]}}", owned.block);
    let root = super::parse::document(&receipt)?;
    let (_, profiles) = super::parse::profiles(&root)?;
    if profiles.len() != 1
        || profiles[0]
            .get("guid")
            .and_then(Value::as_str)
            .and_then(|s| Guid::parse(s).ok())
            .as_ref()
            != Some(&owned.guid)
    {
        return Err(Error::Invalid("invalid ownership receipt"));
    }
    let canonical = owned_block(&owned.guid, &profiles[0])
        .map_err(|_| Error::Invalid("invalid ownership profile"))?;
    if canonical != owned.block
        || !entries.iter().any(|(guid, span)| {
            guid.as_ref() == Some(&owned.guid) && span.start >= start && span.end <= end
        })
    {
        return Err(Error::Invalid(
            "receipt does not own an actual profile entry",
        ));
    }
    let mut result = settings.to_owned();
    result.replace_range(start..end, "");
    list_range(&result)?;
    Ok(result)
}
fn list_range(settings: &str) -> Result<Range<usize>, Error> {
    Ok(list_layout(settings)?.0)
}
type ListLayout = (Range<usize>, Vec<(Option<Guid>, Range<usize>)>);
fn list_layout(settings: &str) -> Result<ListLayout, Error> {
    let trimmed = settings.trim_start_matches('\u{feff}');
    let offset = settings.len() - trimmed.len();
    let value = super::parse::ast_document(trimmed)?;
    let root = value
        .as_object()
        .ok_or(Error::Invalid("settings must be an object"))?;
    let container = root
        .properties
        .iter()
        .find(|p| p.name.as_str() == "profiles")
        .ok_or(Error::Invalid("existing profile list required"))?;
    let array = if let Some(array) = container.value.as_array() {
        array
    } else {
        container
            .value
            .as_object()
            .and_then(|o| o.properties.iter().find(|p| p.name.as_str() == "list"))
            .and_then(|p| p.value.as_array())
            .ok_or(Error::Invalid("existing profile list required"))?
    };
    let mut identities = HashSet::new();
    let mut entries = Vec::new();
    for element in &array.elements {
        let object = element
            .as_object()
            .ok_or(Error::Invalid("profile must be an object"))?;
        let guid = object
            .properties
            .iter()
            .find(|p| p.name.as_str() == "guid")
            .map(|p| {
                Guid::parse(
                    p.value
                        .as_string_lit()
                        .ok_or(Error::Invalid("invalid profile GUID"))?
                        .value
                        .as_ref(),
                )
            })
            .transpose()?;
        if let Some(guid) = &guid
            && !identities.insert(guid.clone())
        {
            return Err(Error::Collision);
        }
        entries.push((guid, element.start() + offset..element.end() + offset));
    }
    Ok((array.start() + offset..array.end() + offset, entries))
}

fn owned_block(guid: &Guid, profile: &Value) -> Result<String, serde_json::Error> {
    Ok(format!(
        "\n// TerminalHUD owned profile {}\n{},\n",
        guid,
        serde_json::to_string_pretty(profile)?
    ))
}
