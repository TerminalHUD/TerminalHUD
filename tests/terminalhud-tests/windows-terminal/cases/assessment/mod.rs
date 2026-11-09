use serde_json::{Value, json};
use terminalhud_windows_terminal::*;
#[path = "evidence.rs"]
mod evidence;
#[path = "inheritance.rs"]
mod inheritance;
#[path = "refusals.rs"]
mod refusals;

const HUD: &str = include_str!("../../fixtures/assessment/hud.jsonc");
const DEFAULTS: &str = include_str!("../../fixtures/assessment/unsafe-defaults.jsonc");
fn guid(suffix: u8) -> Guid {
    super::guid(suffix)
}
fn assess(
    settings: &str,
    fragments: &[SuppliedFragment<'_>],
    requested: u8,
    command: Option<&str>,
) -> ProfileAssessment {
    assess_snapshot(
        SuppliedSnapshot {
            settings_jsonc: settings,
            terminalhud_fragments: fragments,
        },
        &guid(requested),
        command,
    )
}
fn fragment(jsonc: &str) -> SuppliedFragment<'_> {
    SuppliedFragment {
        namespace: "TerminalHUD",
        document_id: "hud.json",
        jsonc,
    }
}
fn property(assessment: &ProfileAssessment, property: Property) -> &PropertyExpectation {
    &assessment
        .selected
        .as_ref()
        .unwrap()
        .properties
        .iter()
        .find(|p| p.property == property)
        .unwrap()
        .expectation
}
fn expected(assessment: &ProfileAssessment, field: Property, raw: Value, provenance: Provenance) {
    let expected = if matches!(provenance, Provenance::Fragment { .. }) {
        PropertyExpectation::Candidate {
            raw,
            provenance,
            unresolved: UnresolvedValue::InboxDefaultsUnresolved,
        }
    } else {
        PropertyExpectation::Supplied { raw, provenance }
    };
    assert_eq!(property(assessment, field), &expected);
}
fn from_fragment() -> Provenance {
    Provenance::Fragment {
        namespace: "TerminalHUD".into(),
        document_id: "hud.json".into(),
    }
}
fn has(assessment: &ProfileAssessment, reason: RefusalReason) -> bool {
    assessment.findings.iter().any(|f| f.reason == reason)
}
