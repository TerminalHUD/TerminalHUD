use super::*;
use terminalhud_core::config::{Candidate, Configuration, ProfileId, ProfileSelection};

#[test]
fn documentation_defaults_and_roundtrip_agree() {
    let documentation = include_str!("../../../../docs/configuration.md");
    let example = documentation
        .split("```json\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    assert_eq!(decode(example).unwrap(), Configuration::default());
    assert_eq!(
        decode(r#"{"version":1}"#).unwrap(),
        Configuration::default()
    );
    let custom = Configuration::validate(Candidate {
        toggle_interaction: "Alt+Ctrl+F24".into(),
        start_with_windows: false,
        active_background_opacity: 100,
        default_hud_profile: ProfileSelection::Id(ProfileId::new("profile:any").unwrap()),
        ..Candidate::default()
    })
    .unwrap();
    assert_eq!(decode(&encode(&custom).unwrap()).unwrap(), custom);
}

#[test]
fn malformed_unknown_duplicate_and_wrong_types_are_rejected() {
    for json in [
        "",
        "{}",
        "[]",
        "null",
        r#"{"version":1} {}"#,
        r#"{"version":1,"version":1}"#,
        r#"{"version":2}"#,
        r#"{"version":1,"extra":true}"#,
        r#"{"version":1,"hotkeys":{"unknown":"x"}}"#,
        r#"{"version":1,"startup":{"startWithWindows":true,"startWithWindows":false}}"#,
        r#"{"version":1,"terminal":{"activeBackgroundOpacity":1.5}}"#,
        r#"{"version":1,"hotkeys":null}"#,
    ] {
        assert!(decode(json).is_err(), "{json}");
    }
    let error = decode(r#"{"version":1,"startup":{"startWithWindows":"yes"}}"#).unwrap_err();
    assert!(
        error.to_string().contains("startup.startWithWindows"),
        "{error}"
    );
}

#[test]
fn fixed_policy_changes_are_rejected_at_the_property() {
    for (section, property, value) in [
        ("display", "monitor", "\"secondary\""),
        ("display", "initialPresentation", "\"windowed\""),
        ("display", "targetCount", "2"),
        ("terminal", "passiveBackgroundOpacity", "1"),
        ("lifecycle", "initialState", "\"passive\""),
        ("lifecycle", "initialState", "\"hidden\""),
        ("lifecycle", "keepRunningWhenTerminalCloses", "false"),
        ("lifecycle", "leaveTerminalOpenOnExit", "false"),
    ] {
        let json = format!(r#"{{"version":1,"{section}":{{"{property}":{value}}}}}"#);
        let error = decode(&json).unwrap_err();
        assert_eq!(error.property, format!("{section}.{property}"));
    }
}

#[test]
fn failed_json_candidates_preserve_current_configuration() {
    let mut current = decode(r#"{"version":1,"terminal":{"activeBackgroundOpacity":0}}"#).unwrap();
    let saved = current.clone();
    for json in [
        "broken",
        r#"{"version":1,"terminal":{"activeBackgroundOpacity":101}}"#,
        r#"{"version":1,"terminal":{"defaultHudProfile":""}}"#,
        r#"{"version":1,"hotkeys":{"toggleInteraction":"Win+Shift+Backtick"}}"#,
    ] {
        assert!(replace_from_json(&mut current, json).is_err());
        assert_eq!(current, saved);
    }
    replace_from_json(&mut current, r#"{"version":1}"#).unwrap();
    assert_eq!(current, Configuration::default());
}
