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
fn direct_empty_unfocused_object_inherits_own_focused_appearance() {
    // Pin: Profile.cpp LayerJson 211-222 and _FinalizeInheritance 239-253 at
    // 7ff11ee107c984e236960ec948b450ccbc10187b; never recursively merge objects.
    let settings = json!({"profiles":{"defaults":{"opacity":72},"list":[{
        "guid":guid(1).as_str(), "unfocusedAppearance":{}
    }]}});
    let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
    expected(
        &result,
        Property::UnfocusedOpacity,
        json!(72),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::UnfocusedUseAcrylic,
        json!(false),
        from_fragment(),
    );
    let result = assess("{}", &[fragment(HUD)], 1, None);
    expected(
        &result,
        Property::UnfocusedOpacity,
        json!(0),
        from_fragment(),
    );
    expected(
        &result,
        Property::UnfocusedColorScheme,
        json!("Campbell"),
        from_fragment(),
    );
}

#[test]
fn fragment_owned_unfocused_inherits_fragment_focused_not_user_defaults() {
    let settings = r#"{"profiles":{"defaults":{"colorScheme":"User scheme"}}}"#;
    let result = assess(settings, &[fragment(HUD)], 1, None);
    expected(
        &result,
        Property::ColorScheme,
        json!("User scheme"),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::UnfocusedColorScheme,
        json!("Campbell"),
        from_fragment(),
    );
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

#[test]
fn omitted_inbox_defaults_keep_fragment_values_and_nested_ownership_as_candidates() {
    let result = assess("{}", &[fragment(HUD)], 1, None);
    for field in [
        Property::Elevate,
        Property::Opacity,
        Property::UnfocusedOpacity,
    ] {
        assert!(matches!(
            property(&result, field),
            PropertyExpectation::Candidate {
                provenance: Provenance::Fragment { .. },
                unresolved: UnresolvedValue::InboxDefaultsUnresolved,
                ..
            }
        ));
    }
    assert!(has(&result, RefusalReason::InboxDefaultsUnresolved));
    let settings = json!({"profiles":{"defaults":{"elevate":false},"list":[{
        "guid":guid(1).as_str(),"opacity":42,"unfocusedAppearance":{}
    }]}});
    let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
    expected(
        &result,
        Property::Elevate,
        json!(false),
        Provenance::UserDefaults,
    );
    expected(
        &result,
        Property::UnfocusedOpacity,
        json!(42),
        Provenance::UserProfile,
    );
    // Missing direct/default values still cross the unsupplied inbox slot.
    assert!(matches!(
        property(&result, Property::UnfocusedUseAcrylic),
        PropertyExpectation::Candidate { .. }
    ));
}
