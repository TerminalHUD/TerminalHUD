use super::*;

#[test]
fn missing_requested_guid_never_selects_the_ordinary_default() {
    let result = assess(DEFAULTS, &[], 1, None);
    assert!(result.selected.is_none());
    assert!(has(&result, RefusalReason::MissingRequestedGuid));
    assert_eq!(
        result.fallback,
        FallbackAssessment::OrdinaryDefaultPossible {
            configured_guid: Some(guid(2))
        }
    );
}

#[test]
fn duplicate_user_guids_and_conflicting_fragment_guids_are_refused() {
    let settings = json!({"profiles":[{"guid":guid(1).as_str()},{"guid":guid(1).as_str()}]});
    assert!(has(
        &assess(&settings.to_string(), &[], 1, None),
        RefusalReason::DuplicateGuid
    ));
    let second = SuppliedFragment {
        document_id: "second.json",
        ..fragment(HUD)
    };
    assert!(has(
        &assess("{}", &[fragment(HUD), second], 1, None),
        RefusalReason::ConflictingFragmentGuid
    ));
    assert!(has(
        &assess("{}", &[fragment(HUD), fragment(HUD)], 1, None),
        RefusalReason::DuplicateFragmentInput
    ));
}

#[test]
fn same_namespace_distinct_documents_with_distinct_guids_are_supported() {
    let second_json = HUD.replace(guid(1).as_str(), guid(2).as_str());
    let second = SuppliedFragment {
        document_id: "second.json",
        ..fragment(&second_json)
    };
    let result = assess("{}", &[fragment(HUD), second], 2, None);
    expected(
        &result,
        Property::Commandline,
        json!("pwsh.exe -NoLogo"),
        Provenance::Fragment {
            namespace: "TerminalHUD".into(),
            document_id: "second.json".into(),
        },
    );
    assert!(!has(&result, RefusalReason::ConflictingFragmentGuid));
}

#[test]
fn unresolved_provider_or_disabled_fragment_cannot_supply_a_resolved_profile() {
    for source in ["Windows.Terminal.Wsl", "TerminalHUD"] {
        let settings = json!({"profiles":[{"guid":guid(1).as_str(),"source":source,"commandline":"explicit.exe"}]});
        let result = assess(&settings.to_string(), &[], 1, None);
        assert!(has(&result, RefusalReason::UnresolvedSource));
        assert_eq!(
            property(&result, Property::Commandline),
            &PropertyExpectation::Unresolved(UnresolvedValue::UnresolvedSource)
        );
    }
    let settings = r#"{"disabledProfileSources":["TerminalHUD"]}"#;
    let result = assess(settings, &[fragment(HUD)], 1, None);
    assert!(result.selected.is_none());
    assert!(has(&result, RefusalReason::DisabledSource));
}

#[test]
fn unresolved_unselected_provider_is_still_reported_and_other_namespaces_are_unsupported() {
    let settings = json!({"profiles":[{"guid":guid(2).as_str(),"source":"Windows.Terminal.Wsl"}]});
    let result = assess(&settings.to_string(), &[fragment(HUD)], 1, None);
    assert!(has(&result, RefusalReason::UnresolvedSource));
    let other = SuppliedFragment {
        namespace: "Other",
        ..fragment(HUD)
    };
    assert!(has(
        &assess("{}", &[other], 1, None),
        RefusalReason::UnsupportedNamespace
    ));
}

#[test]
fn fragment_updates_are_unsupported() {
    let update = HUD.replace("\"name\":", "\"updates\":\"unresolved\",\"name\":");
    assert!(has(
        &assess("{}", &[fragment(&update)], 1, None),
        RefusalReason::UnsupportedUpdate
    ));
}
