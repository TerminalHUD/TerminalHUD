use super::super::{
    properties,
    types::{AssessmentFinding, Provenance, RefusalReason, finding},
};
use super::{Entry, Layer};
use crate::Guid;
use serde_json::{Map, Value};

pub(super) fn parse(
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

pub(super) fn parse_guid(value: &Value, label: &str) -> Result<Guid, AssessmentFinding> {
    value
        .as_str()
        .and_then(|s| Guid::parse(s).ok())
        .filter(|guid| guid.bare() != "00000000-0000-0000-0000-000000000000")
        .ok_or_else(|| finding(RefusalReason::InvalidField, label))
}
pub(super) fn reject_updates(
    values: &Map<String, Value>,
    label: &str,
) -> Result<(), AssessmentFinding> {
    if values.contains_key("updates") {
        Err(finding(RefusalReason::UnsupportedUpdate, label))
    } else {
        Ok(())
    }
}
