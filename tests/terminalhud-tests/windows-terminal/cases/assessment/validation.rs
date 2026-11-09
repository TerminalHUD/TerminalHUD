use super::*;

#[test]
fn duplicate_keys_are_found_before_lossy_conversion_including_escaped_keys() {
    for text in [
        r#"{"profiles":[], "profiles":[]}"#,
        r#"{"profiles":{"defaults":{"elevate":true,"\u0065levate":false}}}"#,
        r#"{"extra":[{"nested":1,"nested":2}]}"#,
    ] {
        let result = assess(text, &[fragment(HUD)], 1, None);
        assert!(result.selected.is_none());
        assert!(has(&result, RefusalReason::DuplicateJsonKey));
    }
    let malformed_fragment = HUD.replace(
        "\"elevate\": false",
        "\"elevate\": true, \"elevate\": false",
    );
    assert!(has(
        &assess("{}", &[fragment(&malformed_fragment)], 1, None),
        RefusalReason::DuplicateJsonKey
    ));
}

#[test]
fn unsupported_forms_remain_distinct_from_malformed_supported_values() {
    let settings = json!({"profiles":[{"guid":guid(1).as_str(),"elevate":null,
        "colorScheme":{"light":"Campbell","dark":"Campbell"},"unfocusedAppearance":null}]});
    let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
    assert!(has(&result, RefusalReason::UnsupportedForm));
    assert!(!has(&result, RefusalReason::InvalidField));
    assert_eq!(
        property(&result, Property::Elevate),
        &PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm)
    );
    assert_eq!(
        property(&result, Property::ColorScheme),
        &PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm)
    );
    assert_eq!(
        property(&result, Property::UnfocusedOpacity),
        &PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm)
    );
}

#[test]
fn json5_and_missing_comma_extensions_are_not_accepted_as_jsonc() {
    for text in [
        r#"{"profiles":[] "extra":true}"#,
        "{profiles:[]}",
        "{'profiles':[]}",
        r#"{"extra":0x35}"#,
        r#"{"extra":+35}"#,
        r#"{"extra":.5}"#,
        r#"{"extra":NaN}"#,
        r#"{"extra":"\x41"}"#,
    ] {
        let result = assess(text, &[fragment(HUD)], 1, None);
        assert!(result.selected.is_none(), "{text}");
        assert!(has(&result, RefusalReason::MalformedDocument), "{text}");
    }
}

#[test]
fn malformed_documents_containers_and_default_guids_are_typed() {
    for (text, reason) in [
        ("{", RefusalReason::MalformedDocument),
        ("[]", RefusalReason::InvalidContainer),
        (r#"{"profiles":false}"#, RefusalReason::InvalidContainer),
        (
            r#"{"defaultProfile":"invalid"}"#,
            RefusalReason::InvalidField,
        ),
        (
            r#"{"profiles":[{"name":"Generated GUID unsupported"}]}"#,
            RefusalReason::MissingGuid,
        ),
    ] {
        let result = assess(text, &[], 1, None);
        assert!(result.selected.is_none());
        assert!(has(&result, reason));
    }
}

#[test]
fn malformed_supported_property_values_are_typed() {
    for (field, value) in [
        ("elevate", json!("false")),
        ("opacity", json!(101)),
        ("backgroundImageOpacity", json!(-1)),
        ("connectionType", json!("unknown")),
        ("environment", json!({"VAR":42})),
        ("unfocusedAppearance", json!(false)),
    ] {
        let mut hud: Value = jsonc_parser::parse_to_serde_value(HUD, &Default::default()).unwrap();
        hud["profiles"][0][field] = value;
        let result = assess("{}", &[fragment(&hud.to_string())], 1, None);
        assert!(result.selected.is_none());
        assert!(has(&result, RefusalReason::InvalidField));
    }
}
