use super::findings::record;
use super::inheritance::{lookup, unfocused_value};
use super::input::ParsedSnapshot;
use super::properties::{APPEARANCE, LAUNCH};
use super::types::*;
use crate::Guid;
use serde_json::Value;

pub(super) fn selected(
    snapshot: &ParsedSnapshot,
    guid: &Guid,
    findings: &mut Vec<AssessmentFinding>,
) -> Option<ExpectedProfile> {
    let user = snapshot.user.iter().find(|entry| &entry.guid == guid);
    let fragment = snapshot.fragments.iter().find(|entry| &entry.guid == guid);
    let disabled = snapshot.disabled_sources.iter().any(|s| s == "TerminalHUD");
    let fragment = fragment.filter(|_| !disabled);
    if user.is_none() && fragment.is_none() {
        findings.push(finding(
            RefusalReason::MissingRequestedGuid,
            guid.to_string(),
        ));
        return None;
    }
    let unresolved_source = user
        .and_then(|u| u.layer.values.get("source"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .is_some_and(|source| source != "TerminalHUD" || fragment.is_none());
    let mut layers = Vec::new();
    if let Some(user) = user {
        layers.push(&user.layer);
    }
    layers.push(&snapshot.defaults);
    if let Some(fragment) = fragment {
        layers.push(&fragment.layer);
    }
    let mut properties = Vec::new();
    for &(property, key) in LAUNCH {
        record(
            &mut properties,
            property,
            lookup(&layers, key),
            unresolved_source,
            findings,
        );
    }
    for &(focused, unfocused, key) in APPEARANCE {
        record(
            &mut properties,
            focused,
            lookup(&layers, key),
            unresolved_source,
            findings,
        );
        record(
            &mut properties,
            unfocused,
            unfocused_value(&layers, key),
            unresolved_source,
            findings,
        );
    }
    Some(ExpectedProfile {
        guid: guid.clone(),
        properties,
    })
}
