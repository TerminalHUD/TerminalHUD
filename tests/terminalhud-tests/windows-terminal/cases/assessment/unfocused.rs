use super::*;

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
