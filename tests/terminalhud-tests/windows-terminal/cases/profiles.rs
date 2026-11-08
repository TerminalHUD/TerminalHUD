use super::*;
use serde_json::json;
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
    assert_eq!(profile["elevate"], false);
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
fn dynamic_source_does_not_inherit_a_resolved_command_from_defaults() {
    let settings = json!({"profiles":{"defaults":{"commandline":"unrelated.exe"},"list":[{
      "guid":guid(1).as_str(), "name":"Dynamic source", "source":"Microsoft.WSL"
    }]}});
    let catalog = Catalog::parse(&settings.to_string()).unwrap();
    assert!(matches!(
        catalog.preview(&guid(1), guid(2), 35),
        Err(Error::UnresolvedSource)
    ));
}

#[test]
fn invalid_profile_default_command_never_becomes_a_launchable_shell() {
    let settings = json!({"profiles":{"defaults":{"commandline":"unrelated.exe","source":"unrelated.source"},"list":[{
        "guid":guid(1).as_str(),"name":"No explicit command"
    }]}});
    let catalog = Catalog::parse(&settings.to_string()).unwrap();
    assert!(matches!(
        catalog.preview(&guid(1), guid(2), 35),
        Err(Error::UnresolvedSource)
    ));
}

#[test]
fn profile_defaults_never_invent_profile_identity_but_keep_valid_defaults() {
    let settings = json!({"profiles":{"defaults":{
        "guid":guid(9).as_str(),"name":"Invented","source":"unrelated.source","commandline":"unrelated.exe",
        "font":{"face":"Consolas","size":18},"startingDirectory":"C:\\fixture"
    },"list":[{}, {"guid":guid(1).as_str(),"name":"Explicit","commandline":"fixture.exe"}]}});
    let catalog = Catalog::parse(&settings.to_string()).unwrap();
    assert_eq!(catalog.unresolved_entries, 1);
    assert_eq!(catalog.profiles.len(), 1);
    let preview = catalog.preview(&guid(1), guid(2), 35).unwrap();
    let profile = &preview.json()["profiles"][0];
    assert_eq!(profile["name"], "HUD — Explicit");
    assert_eq!(profile["commandline"], "fixture.exe");
    assert_eq!(profile["startingDirectory"], "C:\\fixture");
    assert_eq!(profile["font"], json!({"face":"Consolas","size":18}));
    assert!(profile.get("source").is_none());
}
