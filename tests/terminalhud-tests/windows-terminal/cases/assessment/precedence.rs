use super::*;

#[test]
fn user_defaults_override_fragment_safety_and_appearance() {
    let result = assess(DEFAULTS, &[fragment(HUD)], 1, None);
    expected(
        &result,
        Property::Elevate,
        json!(true),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::ConnectionType,
        json!(guid(3).as_str()),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::Opacity,
        json!(90),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::UseAcrylic,
        json!(true),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::UnfocusedOpacity,
        json!(80),
        Provenance::UserDefaults,
    );
    // A defaults-owned unfocused object inherits defaults-focused useAcrylic,
    // rather than the fragment's lower-priority unfocused false.
    expected(
        &result,
        Property::UnfocusedUseAcrylic,
        json!(true),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::Environment,
        json!({"HUD_FIXTURE":"defaults"}),
        Provenance::UserDefaults,
    );
    assert!(has(&result, RefusalReason::ElevatedProfile));
    assert!(has(&result, RefusalReason::NonzeroConnectionType));
}

#[test]
fn direct_same_guid_fields_precede_defaults_and_fragment() {
    let mut settings: Value =
        jsonc_parser::parse_to_serde_value(DEFAULTS, &Default::default()).unwrap();
    settings["profiles"]["list"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "guid": guid(1).as_str(), "source":"TerminalHUD", "elevate":false,
            "commandline":"custom.exe", "connectionType":"{00000000-0000-0000-0000-000000000000}",
            "opacity":45, "environment":{"DIRECT":"only"}
        }));
    let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
    for (field, value) in [
        (Property::Elevate, json!(false)),
        (Property::Opacity, json!(45)),
        (Property::Commandline, json!("custom.exe")),
        (Property::Environment, json!({"DIRECT":"only"})),
    ] {
        expected(&result, field, value, Provenance::UserProfile);
    }
    assert!(!has(&result, RefusalReason::ElevatedProfile));
    assert!(!has(&result, RefusalReason::NonzeroConnectionType));
    assert!(!has(&result, RefusalReason::UnresolvedSource));
}

#[test]
fn invalid_default_identity_and_command_keys_are_cleared_without_fabricated_command() {
    let settings = json!({"profiles":{"defaults":{
        "guid":false,"name":false,"source":false,"commandline":"never-inherited.exe"
    },"list":[{"guid":guid(1).as_str(),"name":"Explicit"}]}});
    let result = assess(&settings.to_string(), &[], 1, None);
    assert_eq!(
        property(&result, Property::Commandline),
        &PropertyExpectation::Unresolved(UnresolvedValue::MissingBaseLayer)
    );
    assert!(has(&result, RefusalReason::MissingProperty));
}
