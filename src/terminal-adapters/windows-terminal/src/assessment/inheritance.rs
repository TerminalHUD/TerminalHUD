use super::input::Layer;
use super::properties;
use super::types::{PropertyExpectation, Provenance, UnresolvedValue};
use serde_json::Value;

pub(super) fn lookup(layers: &[&Layer], key: &str) -> PropertyExpectation {
    layers
        .iter()
        .find_map(|layer| {
            layer
                .values
                .get(key)
                .map(|value| supplied(layer, key, value))
                .or_else(|| legacy_opacity(&layer.values, key))
        })
        .unwrap_or(PropertyExpectation::Unresolved(
            UnresolvedValue::MissingBaseLayer,
        ))
}
fn supplied(layer: &Layer, key: &str, value: &Value) -> PropertyExpectation {
    if properties::supported(key, value) {
        match &layer.provenance {
            Provenance::Fragment { .. } => PropertyExpectation::Candidate {
                raw: value.clone(),
                provenance: layer.provenance.clone(),
                unresolved: UnresolvedValue::InboxDefaultsUnresolved,
            },
            _ => PropertyExpectation::Supplied {
                raw: value.clone(),
                provenance: layer.provenance.clone(),
            },
        }
    } else {
        PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm)
    }
}

pub(super) fn unfocused_value(layers: &[&Layer], key: &str) -> PropertyExpectation {
    // Profile.cpp LayerJson (211-222): first owning unfocused object wins. Its
    // missing fields inherit its OWN focused appearance, not other unfocused objects.
    for (index, layer) in layers.iter().enumerate() {
        if let Some(value) = layer.values.get("unfocusedAppearance") {
            let Some(object) = value.as_object() else {
                return PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm);
            };
            if let Some(value) = object.get(key) {
                return supplied(layer, key, value);
            }
            if let Some(unresolved) = legacy_opacity(object, key) {
                return unresolved;
            }
            return match layer.provenance {
                Provenance::UserProfile => lookup(&layers[index..], key),
                // Defaults and fragments have no other supplied focused parents.
                _ => lookup(&[layer], key),
            };
        }
    }
    PropertyExpectation::Unresolved(UnresolvedValue::MissingBaseLayer)
}

fn legacy_opacity(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Option<PropertyExpectation> {
    // AppearanceConfig.cpp layers acrylicOpacity before opacity into the same
    // setting. Do not skip a higher-priority legacy value to use a lower modern value.
    (key == "opacity" && object.contains_key("acrylicOpacity")).then_some(
        PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm),
    )
}
