use serde_json::{Map, Value, json};

/// Normalize aliases in each layer before precedence is applied. Terminal treats
/// font fields separately, but maps such as axes/features/environment atomically.
pub(super) fn normalize(mut values: Map<String, Value>) -> Map<String, Value> {
    if !values.contains_key("font") {
        let mut font = Map::new();
        for (old, new) in [
            ("fontFace", "face"),
            ("fontSize", "size"),
            ("fontWeight", "weight"),
        ] {
            if let Some(value) = values.get(old) {
                font.insert(new.into(), value.clone());
            }
        }
        if !font.is_empty() {
            values.insert("font".into(), Value::Object(font));
        }
    }
    for key in ["fontFace", "fontSize", "fontWeight"] {
        values.remove(key);
    }
    if !values.contains_key("opacity")
        && let Some(value) = values.get("acrylicOpacity").and_then(Value::as_f64)
    {
        values.insert("opacity".into(), json!(value * 100.0));
    }
    values.remove("acrylicOpacity");
    if let Some(Value::String(name)) = values.get("colorScheme") {
        values.insert("colorScheme".into(), json!({"dark":name,"light":name}));
    }
    for (old, new) in [
        ("experimental.autoMarkPrompts", "autoMarkPrompts"),
        ("experimental.showMarksOnScrollbar", "showMarksOnScrollbar"),
        (
            "experimental.rightClickContextMenu",
            "rightClickContextMenu",
        ),
    ] {
        if !values.contains_key(new)
            && let Some(value) = values.get(old)
        {
            values.insert(new.into(), value.clone());
        }
        values.remove(old);
    }
    // hstring and IMediaResource converters consume null as an explicit empty
    // value. Other OptionalConverter scalars let null unset this layer.
    for key in [
        "name",
        "source",
        "commandline",
        "startingDirectory",
        "tabTitle",
        "padding",
        "icon",
        "backgroundImage",
        "experimental.pixelShaderPath",
        "experimental.pixelShaderImagePath",
        "answerbackMessage",
        "dragDropDelimiter",
        "bellSound",
    ] {
        if values.get(key).is_some_and(Value::is_null) {
            values.insert(key.into(), json!(""));
        }
    }
    if let Some(Value::Object(font)) = values.get_mut("font") {
        for key in ["face", "cellWidth", "cellHeight"] {
            if font.get(key).is_some_and(Value::is_null) {
                font.insert(key.into(), json!(""));
            }
        }
    }
    if let Some(Value::Object(schemes)) = values.get_mut("colorScheme") {
        for key in ["dark", "light"] {
            if schemes.get(key).is_some_and(Value::is_null) {
                schemes.insert(key.into(), json!(""));
            }
        }
    }
    if let Some(Value::Object(environment)) = values.get_mut("environment") {
        for value in environment.values_mut() {
            if value.is_null() {
                *value = json!("");
            }
        }
    }
    // Most native properties are std::optional<T>: JSON null unsets the layer,
    // allowing inheritance. Nullable color settings deliberately retain null.
    values.retain(|key, value| !value.is_null() || !optional_property(key));
    if let Some(Value::Object(font)) = values.get_mut("font") {
        font.retain(|key, value| {
            !value.is_null()
                || !matches!(
                    key.as_str(),
                    "face"
                        | "size"
                        | "weight"
                        | "axes"
                        | "features"
                        | "builtinGlyphs"
                        | "colorGlyphs"
                        | "cellWidth"
                        | "cellHeight"
                )
        });
    }
    if let Some(Value::Object(schemes)) = values.get_mut("colorScheme") {
        schemes.retain(|key, value| !value.is_null() || !matches!(key.as_str(), "dark" | "light"));
    }
    values
}
fn optional_property(key: &str) -> bool {
    matches!(
        key,
        "name"
            | "source"
            | "hidden"
            | "font"
            | "colorScheme"
            | "opacity"
            | "cursorShape"
            | "cursorHeight"
            | "backgroundImageOpacity"
            | "backgroundImageStretchMode"
            | "backgroundImageAlignment"
            | "backgroundImage"
            | "intenseTextStyle"
            | "adjustIndistinguishableColors"
            | "useAcrylic"
            | "experimental.retroTerminalEffect"
            | "experimental.pixelShaderPath"
            | "experimental.pixelShaderImagePath"
            | "historySize"
            | "snapOnInput"
            | "altGrAliasing"
            | "commandline"
            | "scrollbarState"
            | "antialiasingMode"
            | "startingDirectory"
            | "icon"
            | "suppressApplicationTitle"
            | "connectionType"
            | "closeOnExit"
            | "tabTitle"
            | "bellStyle"
            | "environment"
            | "rightClickContextMenu"
            | "bellSound"
            | "elevate"
            | "autoMarkPrompts"
            | "showMarksOnScrollbar"
            | "experimental.repositionCursorWithMouse"
            | "compatibility.reloadEnvironmentVariables"
            | "pathTranslationStyle"
            | "padding"
    )
}

pub(super) fn overlay(base: &mut Map<String, Value>, layer: &Map<String, Value>) {
    for (key, value) in layer {
        // Only these containers have individual inheritable child properties.
        if matches!(key.as_str(), "font" | "colorScheme")
            && let (Some(Value::Object(existing)), Value::Object(update)) =
                (base.get_mut(key), value)
        {
            existing.extend(update.clone());
            continue;
        }
        base.insert(key.clone(), value.clone());
    }
}

/// Layers are ordered most-important first. An unfocused object inherits the
/// focused appearance of its own owner, not another layer's unfocused object.
pub(super) fn effective(layers: &[Map<String, Value>]) -> Map<String, Value> {
    let mut result = Map::new();
    for layer in layers.iter().rev() {
        overlay(&mut result, layer);
    }
    if let Some((index, value)) = layers
        .iter()
        .enumerate()
        .find_map(|(index, layer)| layer.get("unfocusedAppearance").map(|value| (index, value)))
        && (value.is_object() || value.is_null())
    {
        let mut focused = appearance_defaults();
        // The user profile inherits all parents. Other parent layers are independent.
        let owners = if index == 0 {
            &layers[index..]
        } else if index == 1 {
            // User defaults inherit inbox defaults; keep that owner chain intact.
            &layers[index..index + 2]
        } else {
            &layers[index..index + 1]
        };
        for owner in owners.iter().rev() {
            overlay(&mut focused, owner);
        }
        focused.remove("unfocusedAppearance");
        // Appearance settings only; launch/font/profile properties do not belong here.
        focused.retain(|key, _| appearance_key(key));
        if let Some(unfocused) = value.as_object() {
            overlay(&mut focused, &normalize(unfocused.clone()));
        }
        result.insert("unfocusedAppearance".into(), Value::Object(focused));
    }
    result
}
fn appearance_key(key: &str) -> bool {
    matches!(
        key,
        "foreground"
            | "background"
            | "selectionBackground"
            | "cursorColor"
            | "colorScheme"
            | "opacity"
            | "useAcrylic"
            | "backgroundImage"
            | "backgroundImageOpacity"
            | "backgroundImageStretchMode"
            | "backgroundImageAlignment"
            | "cursorShape"
            | "cursorHeight"
            | "intenseTextStyle"
            | "adjustIndistinguishableColors"
            | "experimental.retroTerminalEffect"
            | "experimental.pixelShaderPath"
            | "experimental.pixelShaderImagePath"
    )
}

// Terminal AppearanceConfig.h / MTSMSettings.h built-in values. These are needed
// only when materializing an unfocused parent with no inherited focused field;
// leaving them absent would incorrectly inherit the new HUD profile's focus.
fn appearance_defaults() -> Map<String, Value> {
    json!({"foreground":null,"background":null,"selectionBackground":null,"cursorColor":null,
        "colorScheme":{"dark":"Campbell","light":"Campbell"},"opacity":100,
        "cursorShape":"bar","cursorHeight":25,"backgroundImageOpacity":1.0,
        "backgroundImageStretchMode":"uniformToFill","backgroundImageAlignment":"center",
        "backgroundImage":"","intenseTextStyle":"bright","adjustIndistinguishableColors":"automatic",
        "useAcrylic":false,"experimental.retroTerminalEffect":false,"experimental.pixelShaderPath":"","experimental.pixelShaderImagePath":""}).as_object().unwrap().clone()
}
