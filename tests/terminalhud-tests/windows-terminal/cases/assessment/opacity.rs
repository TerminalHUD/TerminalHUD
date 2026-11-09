use super::*;

#[test]
fn legacy_focused_and_unfocused_opacity_never_fall_through_to_lower_modern_values() {
    // AppearanceConfig.cpp:124-125 at 7ff11ee loads acrylicOpacity before opacity.
    for direct in [
        json!({"acrylicOpacity":0.9}),
        json!({"unfocusedAppearance":{"acrylicOpacity":0.2}}),
    ] {
        let mut user = direct;
        user["guid"] = json!(guid(1).as_str());
        let settings = json!({"profiles":{"defaults":{"opacity":35},"list":[user]}});
        let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
        let field = if settings["profiles"]["list"][0]
            .get("unfocusedAppearance")
            .is_some()
        {
            Property::UnfocusedOpacity
        } else {
            Property::Opacity
        };
        assert_eq!(
            property(&result, field),
            &PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm)
        );
        assert!(has(&result, RefusalReason::UnsupportedForm));
    }
    let settings = json!({"profiles":{"defaults":{"acrylicOpacity":0.9,"unfocusedAppearance":{}},"list":[{"guid":guid(1).as_str()}]}});
    let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
    for field in [Property::Opacity, Property::UnfocusedOpacity] {
        assert_eq!(
            property(&result, field),
            &PropertyExpectation::Unresolved(UnresolvedValue::UnsupportedForm)
        );
    }
}

#[test]
fn same_layer_modern_opacity_and_higher_modern_values_supersede_valid_legacy_aliases() {
    let settings = json!({"profiles":{"defaults":{"acrylicOpacity":0.9},"list":[{
        "guid":guid(1).as_str(),"opacity":45,"acrylicOpacity":0.7,
        "unfocusedAppearance":{"opacity":0,"acrylicOpacity":0.2}
    }]}});
    let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
    expected(
        &result,
        Property::Opacity,
        json!(45),
        Provenance::UserProfile,
    );
    expected(
        &result,
        Property::UnfocusedOpacity,
        json!(0),
        Provenance::UserProfile,
    );
    let settings = json!({"profiles":{"defaults":{"opacity":25,"acrylicOpacity":0.9,"unfocusedAppearance":{}},"list":[{"guid":guid(1).as_str()}]}});
    let result = assess(&settings.to_string(), &[], 1, None);
    expected(
        &result,
        Property::UnfocusedOpacity,
        json!(25),
        Provenance::UserDefaults,
    );
}

#[test]
fn fractional_modern_opacity_is_invalid_in_every_layer_but_integral_real_is_supported() {
    // Terminal's IntAsFloatPercentConversionTrait uses jsoncpp 1.9.5 isUInt,
    // which accepts integral reals (35.0 / 3.5e1), but not fractional percentages.
    for settings in [
        json!({"profiles":{"defaults":{"opacity":35.5}}}),
        json!({"profiles":[{"guid":guid(1).as_str(),"opacity":35.5}]}),
        json!({"profiles":[{"guid":guid(1).as_str(),"unfocusedAppearance":{"opacity":35.5}}]}),
    ] {
        let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
        assert!(result.selected.is_none());
        assert!(has(&result, RefusalReason::InvalidField));
    }
    for text in ["35.0", "3.5e1", "0", "100"] {
        let settings = format!(
            r#"{{"profiles":[{{"guid":"{}","opacity":{text}}}]}}"#,
            guid(1)
        );
        let result = assess(&settings, &[], 1, None);
        assert!(matches!(
            property(&result, Property::Opacity),
            PropertyExpectation::Supplied { .. }
        ));
    }
    let hud = HUD.replace("\"opacity\": 35", "\"opacity\": 35.5");
    assert!(has(
        &assess("{}", &[fragment(&hud)], 1, None),
        RefusalReason::InvalidField
    ));
}

#[test]
fn malformed_legacy_alias_is_unsupported_even_beside_a_modern_value() {
    let settings =
        json!({"profiles":[{"guid":guid(1).as_str(),"opacity":35,"acrylicOpacity":"unknown"}]});
    let result = assess(&settings.to_string(), &[], 1, None);
    assert!(result.selected.is_none());
    assert!(has(&result, RefusalReason::UnsupportedForm));
}
