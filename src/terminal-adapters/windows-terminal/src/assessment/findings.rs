use super::input::ParsedSnapshot;
use super::types::*;
use crate::Guid;
use serde_json::Value;

pub(super) fn record(
    properties: &mut Vec<ExpectedProperty>,
    property: Property,
    mut expectation: PropertyExpectation,
    unresolved_source: bool,
    findings: &mut Vec<AssessmentFinding>,
) {
    if unresolved_source {
        expectation = PropertyExpectation::Unresolved(UnresolvedValue::UnresolvedSource);
    }
    if matches!(expectation, PropertyExpectation::Candidate { .. }) {
        findings.push(finding(
            RefusalReason::InboxDefaultsUnresolved,
            format!("{property:?}"),
        ));
    }
    let reason = match &expectation {
        PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm) => {
            Some(RefusalReason::UnsupportedForm)
        }
        PropertyExpectation::Unresolved(_) => Some(RefusalReason::MissingProperty),
        PropertyExpectation::Supplied {
            raw: Value::Bool(true),
            ..
        }
        | PropertyExpectation::Candidate {
            raw: Value::Bool(true),
            ..
        } if property == Property::Elevate => Some(RefusalReason::ElevatedProfile),
        PropertyExpectation::Supplied { raw, .. } | PropertyExpectation::Candidate { raw, .. }
            if property == Property::ConnectionType
                && raw
                    .as_str()
                    .and_then(|s| Guid::parse(s).ok())
                    .is_some_and(|g| g.bare() != "00000000-0000-0000-0000-000000000000") =>
        {
            Some(RefusalReason::NonzeroConnectionType)
        }
        _ => None,
    };
    if let Some(reason) = reason {
        findings.push(finding(reason, format!("{property:?}")));
    }
    properties.push(ExpectedProperty {
        property,
        expectation,
    });
}

pub(super) fn source_findings(snapshot: &ParsedSnapshot, findings: &mut Vec<AssessmentFinding>) {
    if !snapshot.fragments.is_empty()
        && snapshot.disabled_sources.iter().any(|s| s == "TerminalHUD")
    {
        findings.push(finding(RefusalReason::DisabledSource, "TerminalHUD"));
    }
    for user in &snapshot.user {
        if let Some(source) = user
            .layer
            .values
            .get("source")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            let disabled = snapshot.disabled_sources.iter().any(|s| s == source);
            let known = source == "TerminalHUD"
                && !disabled
                && snapshot.fragments.iter().any(|f| f.guid == user.guid);
            if !known {
                findings.push(finding(
                    if disabled {
                        RefusalReason::DisabledSource
                    } else {
                        RefusalReason::UnresolvedSource
                    },
                    &user.layer.location,
                ));
            }
        }
    }
}
