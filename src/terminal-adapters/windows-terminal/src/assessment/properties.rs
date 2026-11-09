use super::types::{AssessmentFinding, Property, RefusalReason, finding};
use crate::Guid;
use serde_json::{Map, Value};

pub(super) const LAUNCH: &[(Property, &str)] = &[
    (Property::Commandline, "commandline"),
    (Property::StartingDirectory, "startingDirectory"),
    (Property::Environment, "environment"),
    (Property::Elevate, "elevate"),
    (Property::ConnectionType, "connectionType"),
    (Property::Hidden, "hidden"),
];
pub(super) const APPEARANCE: &[(Property, Property, &str)] = &[
    (Property::Opacity, Property::UnfocusedOpacity, "opacity"),
    (
        Property::UseAcrylic,
        Property::UnfocusedUseAcrylic,
        "useAcrylic",
    ),
    (
        Property::BackgroundImage,
        Property::UnfocusedBackgroundImage,
        "backgroundImage",
    ),
    (
        Property::BackgroundImageOpacity,
        Property::UnfocusedBackgroundImageOpacity,
        "backgroundImageOpacity",
    ),
    (
        Property::ColorScheme,
        Property::UnfocusedColorScheme,
        "colorScheme",
    ),
];

pub(super) fn validate(profile: &Map<String, Value>, label: &str) -> Result<(), AssessmentFinding> {
    for (_, key) in LAUNCH {
        if let Some(value) = profile.get(*key) {
            validate_value(key, value, label)?;
        }
    }
    validate_appearance(profile, label)?;
    if let Some(value) = profile.get("unfocusedAppearance") {
        match value {
            Value::Object(object) => {
                validate_appearance(object, &format!("{label}.unfocusedAppearance"))?
            }
            Value::Null => {} // Outside the supported subset, retained as unresolved.
            _ => {
                return Err(finding(
                    RefusalReason::InvalidField,
                    format!("{label}.unfocusedAppearance"),
                ));
            }
        }
    }
    for key in ["name", "source"] {
        if profile.get(key).is_some_and(|v| !v.is_string()) {
            return Err(finding(
                RefusalReason::InvalidField,
                format!("{label}.{key}"),
            ));
        }
    }
    Ok(())
}

fn validate_appearance(object: &Map<String, Value>, label: &str) -> Result<(), AssessmentFinding> {
    for (_, _, key) in APPEARANCE {
        if let Some(value) = object.get(*key) {
            validate_value(key, value, label)?;
        }
    }
    Ok(())
}

fn validate_value(key: &str, value: &Value, label: &str) -> Result<(), AssessmentFinding> {
    // Null and structured color schemes are intentionally unresolved, not guessed or coerced.
    if value.is_null() || (key == "colorScheme" && value.is_object()) {
        return Ok(());
    }
    let valid = match key {
        "elevate" | "hidden" | "useAcrylic" => value.is_boolean(),
        "connectionType" => value.as_str().is_some_and(|s| Guid::parse(s).is_ok()),
        "opacity" => value.as_f64().is_some_and(|v| (0.0..=100.0).contains(&v)),
        "backgroundImageOpacity" => value.as_f64().is_some_and(|v| (0.0..=1.0).contains(&v)),
        "environment" => value
            .as_object()
            .is_some_and(|o| o.values().all(Value::is_string)),
        _ => value.is_string(),
    };
    if valid {
        Ok(())
    } else {
        Err(finding(
            RefusalReason::InvalidField,
            format!("{label}.{key}"),
        ))
    }
}

pub(super) fn supported(key: &str, value: &Value) -> bool {
    !value.is_null()
        && (key != "colorScheme" || value.is_string())
        && (key != "commandline" || value.as_str().is_some_and(|s| !s.trim().is_empty()))
}
