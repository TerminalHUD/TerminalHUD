use super::*;
use serde_json::{Value, json};

fn input(defaults: Value, profile: Value) -> String {
    json!({"defaultProfile":guid(1).as_str(), "profiles":{"defaults":defaults,"list":[profile]},
        "theme":"user-theme","actions":[{"command":"paste","keys":"ctrl+v"}],"unknownGlobal":{"keep":true}}).to_string()
}
fn source() -> Value {
    json!({"guid":guid(1).as_str(),"name":"User shell","commandline":"shell.exe --exact-argument"})
}

#[test]
fn owned_profile_preserves_every_supplied_setting_and_only_minimum_hud_exceptions() {
    let mut profile = source();
    profile.as_object_mut().unwrap().extend(json!({
        "font":{"size":18,"axes":{"wght":450}}, "foreground":"#123456", "background":"#987654",
        "cursorColor":"#aabbcc","cursorShape":"vintage","cursorHeight":40,"padding":"1, 2, 3, 4",
        "colorScheme":{"light":"User light"},"backgroundImage":"C:\\images\\wall.png",
        "backgroundImageOpacity":0.6,"useAcrylic":true,"startingDirectory":"%USERPROFILE%\\code",
        "environment":{"USER_SETTING":"kept"},"unknownFuture":{"nested":[{"opaque":true}]},
        "unfocusedAppearance":{"foreground":"#fefefe","unknownUnfocused":{"deep":[1,2]}}
    }).as_object().unwrap().clone());
    let settings = input(
        json!({"font":{"face":"User font","size":12,"features":{"liga":1}},
        "colorScheme":"Default scheme","opacity":77}),
        profile.clone(),
    );
    let original_bytes = settings.as_bytes().to_vec();
    let catalog = Catalog::from_sources(&settings, None, &[], &[]).unwrap();
    let prepared = catalog.prepare(None, guid(9), 35).unwrap();
    assert_eq!(settings.as_bytes(), original_bytes);
    let owned = &prepared.owned_profile;
    assert_eq!(
        owned["font"],
        json!({"face":"User font","size":18,"features":{"liga":1},"axes":{"wght":450}})
    );
    assert_eq!(
        owned["colorScheme"],
        json!({"dark":"Default scheme","light":"User light"})
    );
    for key in [
        "foreground",
        "background",
        "cursorColor",
        "cursorShape",
        "cursorHeight",
        "padding",
        "backgroundImage",
        "backgroundImageOpacity",
        "useAcrylic",
        "startingDirectory",
        "environment",
        "unknownFuture",
    ] {
        assert_eq!(owned[key], profile[key], "{key}");
    }
    assert_eq!(owned["unfocusedAppearance"]["foreground"], "#fefefe");
    assert_eq!(
        owned["unfocusedAppearance"]["unknownUnfocused"],
        profile["unfocusedAppearance"]["unknownUnfocused"]
    );
    assert_eq!(owned["opacity"], 35);
    assert_eq!(owned["unfocusedAppearance"]["opacity"], 0);
    assert_eq!(owned["unfocusedAppearance"]["backgroundImageOpacity"], 0);
    assert_eq!(owned["unfocusedAppearance"]["useAcrylic"], false);
    assert_eq!(prepared.commandline, "shell.exe --exact-argument");
    assert_eq!(prepared.environment["USER_SETTING"], "kept");
    assert_eq!(prepared.source_guid, guid(1));
    assert!(
        !prepared.fragment_compatible,
        "defaults outrank fragment overrides"
    );
    assert!(
        owned.get("theme").is_none(),
        "global settings remain with installed Terminal"
    );
    // At actual user-list precedence the owned entry retains intended values.
    let patch = plan_profile_patch(&settings, owned, None).unwrap();
    let installed = Catalog::parse(&patch.updated).unwrap();
    let next = installed.prepare(Some(&guid(9)), guid(8), 35).unwrap();
    assert_eq!(next.owned_profile["font"], owned["font"]);
    assert_eq!(next.owned_profile["unknownFuture"], owned["unknownFuture"]);
    assert_eq!(
        remove_owned_profile(&patch.updated, &patch.ownership).unwrap(),
        settings
    );
}

#[test]
fn native_layer_order_inbox_provider_fragment_update_defaults_then_inline() {
    let settings = input(
        json!({"font":{"size":14},"padding":"defaults"}),
        json!({"guid":guid(1).as_str(),"name":"Renamed","source":"vendor","foreground":"#ffffff"}),
    );
    let provider = GeneratedProfile {
        source: "vendor",
        profile: json!({"guid":guid(1).as_str(),"name":"Generated", "commandline":"generated.exe --real","font":{"face":"Provider font","size":9}, "padding":"provider","background":"#010203"}),
    };
    let fragment = json!({"profiles":[{"updates":guid(1).as_str(),"font":{"size":10,"weight":"bold"},"padding":"fragment","background":"#112233"}]}).to_string();
    let catalog = Catalog::from_sources(
        &settings,
        None,
        &[ProfileFragment {
            namespace: "extension",
            jsonc: &fragment,
        }],
        &[provider],
    )
    .unwrap();
    let result = catalog.prepare(None, guid(9), 35).unwrap();
    assert_eq!(result.commandline, "generated.exe --real");
    assert_eq!(
        result.owned_profile["font"],
        json!({"face":"Provider font","size":14,"weight":"bold"})
    );
    assert_eq!(result.owned_profile["padding"], "defaults");
    assert_eq!(result.owned_profile["background"], "#112233");
    assert_eq!(result.owned_profile["foreground"], "#ffffff");
    assert_eq!(result.owned_profile["name"], "HUD — Renamed");
    assert!(result.owned_profile.get("source").is_none());
}

#[test]
fn font_aliases_normalize_before_precedence_and_maps_remain_atomic() {
    let settings = input(
        json!({"font":{"face":"Default","size":10,"axes":{"wght":300,"wdth":100},"features":{"liga":1}},"environment":{"LOWER":"value"}}),
        json!({
            "guid":guid(1).as_str(),"name":"Aliases","commandline":"real.exe", "fontFace":"Legacy face","fontSize":17,
            "environment":{"HIGHER":"value"}
        }),
    );
    let result = Catalog::parse(&settings)
        .unwrap()
        .prepare(None, guid(9), 35)
        .unwrap();
    assert_eq!(result.owned_profile["font"]["face"], "Legacy face");
    assert_eq!(result.owned_profile["font"]["size"], 17);
    assert_eq!(result.environment.len(), 1);
    let mut profile = source();
    profile["font"] = json!({"axes":{"wght":550}});
    profile["fontFace"] = json!("Ignored by native font object");
    let settings = input(
        json!({"font":{"face":"Default","axes":{"wdth":100}}}),
        profile,
    );
    let result = Catalog::parse(&settings)
        .unwrap()
        .prepare(None, guid(9), 35)
        .unwrap();
    assert_eq!(result.owned_profile["font"]["face"], "Default");
    assert_eq!(result.owned_profile["font"]["axes"], json!({"wght":550}));
}

#[test]
fn inherited_unfocused_uses_its_own_focus_for_object_and_null() {
    for unfocused in [json!({}), Value::Null] {
        let mut profile = source();
        profile["foreground"] = json!("#ff0000");
        profile["background"] = json!("#aabbcc");
        let settings = input(
            json!({"foreground":"#00ff00","unfocusedAppearance":unfocused}),
            profile,
        );
        let result = Catalog::parse(&settings)
            .unwrap()
            .prepare(None, guid(9), 35)
            .unwrap();
        assert_eq!(result.owned_profile["foreground"], "#ff0000");
        assert_eq!(
            result.owned_profile["unfocusedAppearance"]["foreground"],
            "#00ff00"
        );
        assert_eq!(
            result.owned_profile["unfocusedAppearance"]["background"],
            Value::Null
        );
    }
}

#[test]
fn inbox_default_identity_and_launch_values_are_used_without_fabrication() {
    let inbox = json!({"defaultProfile":guid(1).as_str(),"profiles":[{"guid":guid(1).as_str(),"name":"Inbox", "commandline":"%SystemRoot%\\System32\\cmd.exe","fontFace":"Inbox font","startingDirectory":"%USERPROFILE%"}]}).to_string();
    let settings = "{\"profiles\":{\"list\":[]}}";
    let result = Catalog::from_sources(settings, Some(&inbox), &[], &[])
        .unwrap()
        .prepare(None, guid(9), 35)
        .unwrap();
    assert_eq!(result.source_guid, guid(1));
    assert_eq!(result.commandline, "%SystemRoot%\\System32\\cmd.exe");
    assert_eq!(result.owned_profile["font"]["face"], "Inbox font");
    assert!(
        Catalog::from_sources(settings, None, &[], &[])
            .unwrap()
            .prepare(None, guid(9), 35)
            .is_err()
    );
}

#[test]
fn missing_disabled_ambiguous_and_nonconpty_sources_are_explicit_errors() {
    let mut profile = source();
    profile["source"] = json!("missing.provider");
    assert!(matches!(
        Catalog::parse(&input(json!({}), profile))
            .unwrap()
            .prepare(None, guid(9), 35),
        Err(Error::UnresolvedSource)
    ));
    let mut profile = source();
    profile["connectionType"] = json!(guid(8).as_str());
    assert!(
        Catalog::parse(&input(json!({}), profile))
            .unwrap()
            .prepare(None, guid(9), 35)
            .is_err()
    );
    let mut profile = source();
    profile["hidden"] = json!(true);
    assert!(
        Catalog::parse(&input(json!({}), profile))
            .unwrap()
            .prepare(None, guid(9), 35)
            .is_err()
    );
    assert!(Catalog::parse(r#"{"profiles":{"list":[]},"profiles":[]}"#).is_err());
    assert!(Catalog::parse(r#"{profiles:[]}"#).is_err());
    let settings = json!({"disabledProfileSources":["provider"],"defaultProfile":guid(1).as_str(),"profiles":{"list":[{"guid":guid(1).as_str(),"name":"Disabled","source":"provider"}]}}).to_string();
    let provider = GeneratedProfile {
        source: "provider",
        profile: source(),
    };
    assert!(
        Catalog::from_sources(&settings, None, &[], &[provider])
            .unwrap()
            .prepare(None, guid(9), 35)
            .is_err()
    );
}

#[test]
fn fallback_checks_only_the_defaults_and_fallback_connection_before_bridge_spawn() {
    let settings = input(json!({"elevate":true}), source());
    assert!(matches!(
        Catalog::parse(&settings).unwrap().validate_fallback(),
        Err(Error::ElevatedProfile)
    ));
    let mut profile = source();
    profile["connectionType"] = json!(guid(8).as_str());
    assert!(
        Catalog::parse(&input(json!({}), profile))
            .unwrap()
            .validate_fallback()
            .is_err()
    );
    let settings = input(
        json!({}),
        json!({"guid":guid(1).as_str(),"name":"Unknown source","source":"provider"}),
    );
    assert!(
        Catalog::parse(&settings)
            .unwrap()
            .validate_fallback()
            .is_ok()
    );
    assert!(
        Catalog::parse(&input(json!({"elevate":"wrong type"}), source()))
            .unwrap()
            .validate_fallback()
            .is_err()
    );
}

#[test]
fn wsl_directory_promotion_matches_native_terminal_without_replacing_arguments() {
    let system = r"C:\Windows\System32";
    let planned = plan_wsl_launch(
        r#""C:\Windows\System32\wsl.exe" -d Ubuntu --exec bash -l"#,
        "~",
        system,
    )
    .unwrap();
    assert_eq!(
        planned.commandline,
        r#""C:\Windows\System32\wsl.exe" --cd "~"  -d Ubuntu --exec bash -l"#
    );
    assert_eq!(planned.starting_directory, None);
    assert_eq!(
        plan_wsl_launch("wsl -d Debian", r"//wsl$/Debian/home/user", system)
            .unwrap()
            .commandline,
        r#""wsl" --cd "\\wsl$\Debian\home\user" -d Debian"#
    );
    for command in [
        "pwsh.exe",
        r"C:\other\wsl.exe -d Ubuntu",
        "wsl --cd /root",
        "wsl ~",
        "wsl ~ -d Ubuntu",
    ] {
        assert!(plan_wsl_launch(command, "~", system).is_none(), "{command}");
    }
    assert!(plan_wsl_launch("wsl -d Ubuntu", "", system).is_none());
    assert!(plan_wsl_launch("wsl ~/run.sh", "/home", system).is_some());
}

#[test]
fn inbox_unfocused_owner_is_not_reparented_to_user_defaults() {
    let settings = input(json!({"foreground":"#ff0000"}), source());
    let inbox = json!({"profiles":{"defaults":{"foreground":"#00ff00","unfocusedAppearance":{}},"list":[]}}).to_string();
    let result = Catalog::from_sources(&settings, Some(&inbox), &[], &[])
        .unwrap()
        .prepare(None, guid(9), 35)
        .unwrap();
    assert_eq!(result.owned_profile["foreground"], "#ff0000");
    assert_eq!(
        result.owned_profile["unfocusedAppearance"]["foreground"],
        "#00ff00"
    );
}
#[test]
fn optional_null_inherits_but_nullable_color_and_unknown_null_are_retained() {
    let settings = input(
        json!({}),
        json!({"guid":guid(1).as_str(),"name":"User","source":"vendor","font":{"size":null},
        "foreground":null,"backgroundImage":null,"unknownFuture":null}),
    );
    let fragment = json!({"profiles":[{"guid":guid(1).as_str(),"name":"Vendor","commandline":"vendor.exe",
        "font":{"size":17},"foreground":"#123456","backgroundImage":"C:\\vendor.png","unknownFuture":{"value":1}}]}).to_string();
    let result = Catalog::from_sources(
        &settings,
        None,
        &[ProfileFragment {
            namespace: "vendor",
            jsonc: &fragment,
        }],
        &[],
    )
    .unwrap()
    .prepare(None, guid(9), 35)
    .unwrap();
    assert_eq!(result.owned_profile["font"]["size"], 17);
    assert_eq!(result.owned_profile["foreground"], Value::Null);
    assert_eq!(result.owned_profile["backgroundImage"], "");
    assert_eq!(result.owned_profile["unknownFuture"], Value::Null);
}

#[test]
fn native_null_strings_and_resources_clear_instead_of_inheriting() {
    let mut profile = source();
    profile["font"] = json!({"face":null,"size":null,"cellWidth":null});
    profile["startingDirectory"] = Value::Null;
    profile["icon"] = Value::Null;
    profile["colorScheme"] = json!({"dark":null});
    profile["environment"] = json!({"CLEARED":null});
    let settings = input(
        json!({"font":{"face":"Lower","size":17,"cellWidth":"120%"},"startingDirectory":"C:\\lower","icon":"C:\\lower.ico","colorScheme":"Lower"}),
        profile,
    );
    let result = Catalog::parse(&settings)
        .unwrap()
        .prepare(None, guid(9), 35)
        .unwrap();
    assert_eq!(result.owned_profile["font"]["face"], "");
    assert_eq!(result.owned_profile["font"]["cellWidth"], "");
    assert_eq!(result.owned_profile["font"]["size"], 17);
    assert_eq!(result.owned_profile["icon"], "");
    assert_eq!(result.starting_directory, Some("".into()));
    assert_eq!(
        result.owned_profile["colorScheme"],
        json!({"dark":"","light":"Lower"})
    );
    assert_eq!(result.environment["CLEARED"], "");
}

#[test]
fn environment_reload_policy_preserves_default_explicit_and_inherited_values() {
    for setting in [None, Some(true), Some(false)] {
        let mut profile = source();
        if let Some(value) = setting {
            profile["compatibility.reloadEnvironmentVariables"] = json!(value);
        }
        let result = Catalog::parse(&input(json!({}), profile))
            .unwrap()
            .prepare(None, guid(9), 35)
            .unwrap();
        assert_eq!(result.reload_environment, setting.unwrap_or(true));
    }
    let mut profile = source();
    profile["compatibility.reloadEnvironmentVariables"] = Value::Null;
    let result = Catalog::parse(&input(
        json!({"compatibility.reloadEnvironmentVariables":false}),
        profile,
    ))
    .unwrap()
    .prepare(None, guid(9), 35)
    .unwrap();
    assert!(
        !result.reload_environment,
        "null bool inherits the selected defaults"
    );
    let mut profile = source();
    profile["compatibility.reloadEnvironmentVariables"] = json!("false");
    assert!(
        Catalog::parse(&input(json!({}), profile))
            .unwrap()
            .prepare(None, guid(9), 35)
            .is_err()
    );
}
