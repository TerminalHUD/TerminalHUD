use hudswitch_core::VisibleState;
use serde_json::json;
use terminalhud_windows_terminal::*;

fn guid(suffix: u8) -> Guid {
    Guid::parse(&format!(
        "{{01234567-89ab-4cde-8fab-0123456789{suffix:02x}}}"
    ))
    .unwrap()
}
fn catalog() -> Catalog {
    Catalog::parse(r#"{
      // Deliberately synthetic settings, including a URL and trailing commas.
      "defaultProfile":"{01234567-89ab-4cde-8fab-012345678901}",
      "profiles":{"defaults":{"font":{"face":"Consolas","size":12},"startingDirectory":"%USERPROFILE%"},
        "list":[{"guid":"{01234567-89ab-4cde-8fab-012345678901}","name":"Synthetic shell","commandline":"pwsh.exe -NoLogo","icon":"https://example.test/icon","font":{"size":14},},
        {"guid":"{01234567-89ab-4cde-8fab-012345678902}","name":"Dynamic WSL","source":"Microsoft.WSL"},
        {"guid":"{01234567-89ab-4cde-8fab-012345678903}","name":"Elevated","commandline":"pwsh.exe","elevate":true}]}}
    "#).unwrap()
}
#[test]
fn preview_preserves_resolved_launch_behavior_and_changes_only_new_variant() {
    let catalog = catalog();
    let before = catalog.default().unwrap().clone();
    let preview = catalog.preview(&guid(1), guid(4), 35).unwrap();
    let profile = &preview.json()["profiles"][0];
    assert_eq!(profile["commandline"], "pwsh.exe -NoLogo");
    assert_eq!(profile["startingDirectory"], "%USERPROFILE%");
    assert_eq!(profile["font"], json!({"face":"Consolas","size":14}));
    assert_eq!(profile["opacity"], 35);
    assert_eq!(profile["unfocusedAppearance"]["opacity"], 0);
    assert_eq!(profile["unfocusedAppearance"]["useAcrylic"], false);
    assert_eq!(profile["guid"], guid(4).as_str());
    assert_ne!(profile["name"], before.name);
    assert!(profile.get("updates").is_none());
    assert!(profile.get("source").is_none());
    assert_eq!(catalog.default().unwrap().guid, before.guid);
    assert_eq!(catalog.default().unwrap().name, before.name);
    assert!(preview.utf8().ends_with('\n'));
}
#[test]
fn unresolved_dynamic_elevated_and_colliding_profiles_are_not_guessed() {
    let catalog = catalog();
    assert!(matches!(
        catalog.preview(&guid(2), guid(4), 35),
        Err(Error::UnresolvedSource)
    ));
    assert!(matches!(
        catalog.preview(&guid(3), guid(4), 35),
        Err(Error::ElevatedProfile)
    ));
    assert!(matches!(
        catalog.preview(&guid(1), guid(2), 35),
        Err(Error::Collision)
    ));
    assert!(matches!(
        catalog.preview(&guid(1), guid(4), 101),
        Err(Error::Invalid(_))
    ));
}
#[test]
fn launch_arguments_are_isolated_fullscreen_and_do_not_use_mru_window() {
    let plan = LaunchPlan {
        identity: LaunchIdentity {
            nonce: guid(5),
            profile: guid(4),
        },
        intent: VisibleState::Passive,
        primary_origin: (0, 0),
    };
    let arguments = plan.arguments();
    assert_eq!(
        &arguments[..2],
        ["--window", plan.identity.window_name().as_str()]
    );
    assert!(arguments.contains(&"--fullscreen".into()));
    assert!(arguments.contains(&guid(4).to_string()));
    assert!(arguments.contains(&"--suppressApplicationTitle".into()));
    assert!(!arguments.contains(&"last".into()));
}
#[test]
fn correlation_requires_nonce_profile_new_window_and_verified_owner_ancestry() {
    let identity = LaunchIdentity {
        nonce: guid(5),
        profile: guid(4),
    };
    let proof = SessionProof {
        nonce: guid(5),
        profile: guid(4),
        session: guid(6),
        shell_pid: 30,
    };
    let candidate = WindowCandidate {
        handle: 11,
        process_id: 20,
        class: "CASCADIA_HOSTING_WINDOW_CLASS".into(),
        title: identity.title(),
        existed_before_launch: false,
    };
    assert_eq!(
        correlate(
            &identity,
            &proof,
            &[30, 20],
            std::slice::from_ref(&candidate)
        )
        .unwrap()
        .handle,
        11
    );
    assert!(correlate(&identity, &proof, &[30], std::slice::from_ref(&candidate)).is_err());
    let mut old = candidate.clone();
    old.existed_before_launch = true;
    assert!(correlate(&identity, &proof, &[30, 20], &[old]).is_err());
    assert!(
        correlate(
            &identity,
            &proof,
            &[30, 20],
            &[candidate.clone(), candidate]
        )
        .is_err()
    );
    let mut wrong = proof;
    wrong.nonce = guid(7);
    assert!(correlate(&identity, &wrong, &[30, 20], &[]).is_err());
}
#[test]
fn malformed_identity_and_duplicate_profiles_are_rejected() {
    assert!(Guid::parse("not-a-guid").is_err());
    assert!(Catalog::parse(r#"{"profiles":{"list":[{"guid":"{01234567-89ab-4cde-8fab-012345678901}","name":"one"},{"guid":"{01234567-89ab-4cde-8fab-012345678901}","name":"two"}]}}"#).is_err());
}
