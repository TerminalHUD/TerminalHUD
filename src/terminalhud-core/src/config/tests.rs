use super::*;

#[test]
fn opacity_is_bounded_and_replacement_is_atomic() {
    let mut current = Configuration::default();
    for value in [0, 100] {
        current
            .replace(Candidate {
                active_background_opacity: value,
                ..Candidate::default()
            })
            .unwrap();
        assert_eq!(current.active_background_opacity(), value as u8);
    }
    let saved = current.clone();
    for value in [-1, 101, i64::MIN, i64::MAX] {
        assert!(
            current
                .replace(Candidate {
                    active_background_opacity: value,
                    ..Candidate::default()
                })
                .is_err()
        );
        assert_eq!(current, saved);
    }
}

#[test]
fn shortcut_grammar_is_explicit_and_normalized() {
    for text in ["Win+Backtick", "Ctrl+Alt+A", "Shift+9", "Alt+F1", "Win+F24"] {
        let shortcut = Shortcut::parse(text).unwrap();
        assert_eq!(Shortcut::parse(&shortcut.to_string()).unwrap(), shortcut);
    }
    assert_eq!(
        Shortcut::parse("Shift+Win+Backtick"),
        Shortcut::parse("Win+Shift+Backtick")
    );
    for text in [
        "",
        "Win",
        "Win+Shift",
        "A",
        "Win+Win+A",
        "Win++A",
        "win+A",
        "Win+a",
        "Win+F0",
        "Win+F25",
        "Win+F01",
        "Win+Enter",
        "Win+A+B",
        "Win+ A",
    ] {
        assert!(Shortcut::parse(text).is_err(), "{text}");
    }
    assert!(
        Configuration::validate(Candidate {
            toggle_interaction: "Shift+Win+Backtick".into(),
            ..Candidate::default()
        })
        .is_err()
    );
}

#[test]
fn profile_ids_are_opaque_bounded_and_non_control() {
    assert!(ProfileId::new("opaque:/identity {x}").is_ok());
    assert!(ProfileId::new("x".repeat(256)).is_ok());
    for id in ["", "   ", "auto", "bad\nidentity", "bad\0identity"] {
        assert!(ProfileId::new(id).is_err());
    }
    assert!(ProfileId::new("x".repeat(257)).is_err());
}
