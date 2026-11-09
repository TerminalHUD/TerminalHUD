use super::{decode, replace_from_json};
use terminalhud_core::config::Configuration;

#[test]
fn document_requires_an_object_with_a_version_property() {
    let mut current = Configuration::default();
    let saved = current.clone();
    for json in [
        "[]",
        "[1,{},{},{},{},{}]",
        "[1]",
        "null",
        "true",
        "1",
        "\"text\"",
        "{}",
    ] {
        assert!(replace_from_json(&mut current, json).is_err(), "{json}");
        assert_eq!(current, saved);
    }
}

#[test]
fn every_section_requires_an_object_and_preserves_property_paths() {
    for (section, positional) in [
        ("hotkeys", r#"["Win+Backtick","Win+Shift+Backtick"]"#),
        ("startup", "[true]"),
        ("display", r#"["primary","fullscreen",1]"#),
        ("terminal", r#"[35,0,"auto"]"#),
        ("lifecycle", r#"["active",true,true]"#),
    ] {
        for value in ["[]", positional, "null", "true", "1", "\"text\""] {
            let json = format!(r#"{{"version":1,"{section}":{value}}}"#);
            let error = decode(&json).unwrap_err().to_string();
            assert!(error.contains(section), "{json}: {error}");
            assert!(error.contains("a JSON object"), "{json}: {error}");
        }
        let empty = format!(r#"{{"version":1,"{section}":{{}}}}"#);
        assert_eq!(decode(&empty).unwrap(), Configuration::default());
        let unknown = format!(r#"{{"version":1,"{section}":{{"unknown":true}}}}"#);
        assert!(
            decode(&unknown)
                .unwrap_err()
                .to_string()
                .contains("unknown field")
        );
    }
}
