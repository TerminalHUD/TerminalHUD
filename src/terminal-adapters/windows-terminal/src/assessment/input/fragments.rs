use super::super::{
    document,
    types::{AssessmentFinding, Provenance, RefusalReason, SuppliedFragment, finding},
};
use super::{Entry, entries};
use serde_json::Value;
use std::collections::HashSet;

pub(super) fn parse(supplied: &[SuppliedFragment<'_>]) -> Result<Vec<Entry>, AssessmentFinding> {
    let mut fragments = Vec::new();
    let mut labels = HashSet::new();
    for fragment in supplied {
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
        for entry in entries::parse(list, provenance, &label, true)? {
            if fragments.iter().any(|e: &Entry| e.guid == entry.guid) {
                return Err(finding(
                    RefusalReason::ConflictingFragmentGuid,
                    entry.layer.location,
                ));
            }
            fragments.push(entry);
        }
    }
    Ok(fragments)
}
