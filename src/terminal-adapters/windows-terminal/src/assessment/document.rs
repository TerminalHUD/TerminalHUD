use super::types::{AssessmentFinding, RefusalReason, finding};
use jsonc_parser::ast;
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Inspect the lossless AST first: converting directly to Value discards duplicate keys.
pub(super) fn parse(text: &str, label: &str) -> Result<Map<String, Value>, AssessmentFinding> {
    let text = text.trim_start_matches('\u{feff}');
    let parsed = jsonc_parser::parse_to_ast(text, &Default::default(), &Default::default())
        .map_err(|_| finding(RefusalReason::MalformedDocument, label))?;
    let value = parsed
        .value
        .ok_or_else(|| finding(RefusalReason::MalformedDocument, label))?;
    check_duplicates(&value, label)?;
    let value: Value = jsonc_parser::parse_to_serde_value(text, &Default::default())
        .map_err(|_| finding(RefusalReason::MalformedDocument, label))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| finding(RefusalReason::InvalidContainer, label))
}

fn check_duplicates(value: &ast::Value<'_>, label: &str) -> Result<(), AssessmentFinding> {
    match value {
        ast::Value::Object(object) => {
            let mut keys = HashSet::new();
            for property in &object.properties {
                let key = property.name.as_str();
                let location = format!("{label}.{key}");
                if !keys.insert(key) {
                    return Err(finding(RefusalReason::DuplicateJsonKey, location));
                }
                check_duplicates(&property.value, &location)?;
            }
        }
        ast::Value::Array(array) => {
            for (index, value) in array.elements.iter().enumerate() {
                check_duplicates(value, &format!("{label}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}
