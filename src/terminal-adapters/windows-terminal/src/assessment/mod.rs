//! Bounded expectations from supplied bytes. No discovery, execution, or launch authorization.
mod document;
mod findings;
mod inheritance;
mod input;
mod properties;
mod resolve;
mod types;
use crate::Guid;
use types::finding;
pub use types::{
    AssessmentFinding, ExpectedProfile, ExpectedProperty, FallbackAssessment, ProfileAssessment,
    Property, PropertyExpectation, Provenance, RefusalReason, SuppliedFragment, SuppliedSnapshot,
    UnresolvedValue,
};

/// Assess explicit GUID profiles plus independent TerminalHUD fragment profiles.
/// The optional command line selects which fallback branches are possible, never a fallback winner.
/// Every result retains unproved runtime, catalog, ownership, inventory and freshness evidence.
pub fn assess_snapshot(
    snapshot: SuppliedSnapshot<'_>,
    requested_guid: &Guid,
    fallback_commandline: Option<&str>,
) -> ProfileAssessment {
    let mut findings = [
        RefusalReason::FallbackUnproved,
        RefusalReason::InventoryCompletenessUnproved,
        RefusalReason::InputFreshnessUnproved,
        RefusalReason::RuntimeRecognitionUnproved,
        RefusalReason::LoadedCatalogUnproved,
        RefusalReason::DedicatedProcessUnproved,
        RefusalReason::LaunchEnvironmentUnproved,
    ]
    .into_iter()
    .map(|reason| finding(reason, "supplied snapshot"))
    .collect::<Vec<_>>();
    let parsed = input::ParsedSnapshot::parse(snapshot);
    let fallback = if fallback_commandline.is_some_and(|s| !s.is_empty()) {
        FallbackAssessment::CommandMatchOrProfileDefaultsUnresolved
    } else {
        FallbackAssessment::OrdinaryDefaultPossible {
            configured_guid: parsed.as_ref().ok().and_then(|p| p.default_profile.clone()),
        }
    };
    let selected = match parsed {
        Ok(parsed) => {
            findings::source_findings(&parsed, &mut findings);
            resolve::selected(&parsed, requested_guid, &mut findings)
        }
        Err(error) => {
            findings.push(error);
            None
        }
    };
    ProfileAssessment {
        selected,
        fallback,
        findings,
    }
}
