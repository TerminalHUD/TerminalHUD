use super::*;
fn settings() -> (Guid, Value) {
    let profile = Guid::parse("e8cff6f0-cd23-4e9d-b65b-58ff2e11c4d6").unwrap();
    let appearance = serde_json::json!({
        "elevate":false, "connectionType":"{00000000-0000-0000-0000-000000000000}",
        "commandline":"bridge --session-bridge", "opacity":35, "useAcrylic":false,
        "suppressApplicationTitle":true, "unfocusedAppearance":{"opacity":0,"useAcrylic":false}
    });
    let mut explicit = appearance.clone();
    explicit["guid"] = profile.as_str().into();
    let value = serde_json::json!({"defaultProfile":profile.as_str(), "startupActions":"", "firstWindowPreference":"defaultProfile",
        "profiles":{"defaults":appearance,"list":[explicit]}, "disabledProfileSources":namespaces::DISABLED});
    (profile, value)
}
#[test]
fn unsafe_fallback_elevation_connection_sources_and_appearance_fail_before_spawn() {
    let (profile, good) = settings();
    validate_settings(&good, &profile, "bridge --session-bridge").unwrap();
    for pointer in [
        "/profiles/defaults/elevate",
        "/profiles/list/0/elevate",
        "/profiles/defaults/commandline",
        "/profiles/list/0/connectionType",
        "/profiles/list/0/opacity",
        "/profiles/list/0/unfocusedAppearance/opacity",
        "/startupActions",
        "/disabledProfileSources",
    ] {
        let mut unsafe_settings = good.clone();
        *unsafe_settings.pointer_mut(pointer).unwrap() = Value::Null;
        assert!(
            validate_settings(&unsafe_settings, &profile, "bridge --session-bridge").is_err(),
            "{pointer}"
        );
    }
    let mut dynamic = good.clone();
    dynamic["profiles"]["list"][0]["source"] = "dynamic".into();
    assert!(validate_settings(&dynamic, &profile, "bridge --session-bridge").is_err());
    assert!(crate::windows::development::run_bridge(&[]).is_err());
}
