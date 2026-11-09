use super::*;

#[test]
fn bridge_command_reports_branches_without_inventing_a_safe_match() {
    // Even a visible matching prefix can be elevated, and runtime normalization,
    // the complete AllProfiles catalog and the existing host cache are unavailable.
    for requested in [1, 99] {
        let result = assess(
            DEFAULTS,
            &[fragment(HUD)],
            requested,
            Some("pwsh.exe bridge.ps1"),
        );
        assert_eq!(
            result.fallback,
            FallbackAssessment::CommandMatchOrProfileDefaultsUnresolved
        );
        assert!(has(&result, RefusalReason::FallbackUnproved));
    }
    let result = assess("{}", &[fragment(HUD)], 99, Some("pwsh.exe bridge.ps1"));
    assert!(result.selected.is_none());
}

#[test]
fn all_results_keep_unproved_runtime_catalog_and_dedicated_process_evidence() {
    for result in [
        assess("{}", &[fragment(HUD)], 1, None),
        assess("{}", &[fragment(HUD)], 99, None),
        assess("{", &[], 1, None),
    ] {
        for reason in [
            RefusalReason::RuntimeRecognitionUnproved,
            RefusalReason::LoadedCatalogUnproved,
            RefusalReason::DedicatedProcessUnproved,
            RefusalReason::InventoryCompletenessUnproved,
            RefusalReason::InputFreshnessUnproved,
            RefusalReason::FallbackUnproved,
            RefusalReason::LaunchEnvironmentUnproved,
        ] {
            assert!(has(&result, reason));
        }
    }
}

#[test]
fn changed_supplied_inputs_cannot_establish_freshness_or_cached_host_recognition() {
    let old = assess("{}", &[fragment(HUD)], 1, None);
    let changed_json = HUD.replace("\"opacity\": 35", "\"opacity\": 65");
    let changed = assess("{}", &[fragment(&changed_json)], 1, None);
    assert_ne!(old.selected, changed.selected);
    for result in [old, changed] {
        assert!(has(&result, RefusalReason::InputFreshnessUnproved));
        assert!(has(&result, RefusalReason::LoadedCatalogUnproved));
        assert!(has(&result, RefusalReason::DedicatedProcessUnproved));
    }
}

#[test]
fn raw_environment_paths_and_commands_are_never_expanded_or_executed() {
    let result = assess("{}", &[fragment(HUD)], 1, None);
    expected(
        &result,
        Property::StartingDirectory,
        json!("%USERPROFILE%"),
        from_fragment(),
    );
    assert!(has(&result, RefusalReason::LaunchEnvironmentUnproved));
    let result = assess(
        r#"{"profiles":[{"guid":"{01234567-89ab-4cde-8fab-012345678901}"}]}"#,
        &[],
        1,
        None,
    );
    assert_eq!(
        property(&result, Property::Elevate),
        &PropertyExpectation::Unresolved(UnresolvedValue::MissingBaseLayer)
    );
    assert_eq!(
        property(&result, Property::ConnectionType),
        &PropertyExpectation::Unresolved(UnresolvedValue::MissingBaseLayer)
    );
}
