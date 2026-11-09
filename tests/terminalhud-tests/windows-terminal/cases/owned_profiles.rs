use super::*;
use serde_json::json;
fn owned() -> serde_json::Value {
    json!({"guid":guid(9).as_str(),"name":"HUD fixture","commandline":"fixture.exe","opacity":35})
}
#[test]
fn edits_round_trip_every_ordinary_byte_for_bom_crlf_comments_and_legacy_arrays() {
    for settings in [
        "\u{feff}{\r\n// retained\r\n\"profiles\":{\"defaults\":{},\"list\":[/*before*/{\"name\":\"ordinary\"},/*after*/]},\r\n\"theme\":\"user\",\r\n}",
        r#"{"profiles": [{"name":"ordinary"} /*no comma*/],"unknown":{"value":42}}"#,
        r#"{"profiles":{"list":[]}}"#,
    ] {
        let patch = plan_profile_patch(settings, &owned(), None).unwrap();
        assert_eq!(patch.original, settings);
        assert_eq!(
            remove_owned_profile(&patch.updated, &patch.ownership).unwrap(),
            settings
        );
        let persisted = OwnedProfileEntry::from_json(&patch.ownership.to_json()).unwrap();
        let mut replacement = owned();
        replacement["opacity"] = json!(51);
        let updated = plan_profile_patch(&patch.updated, &replacement, Some(&persisted)).unwrap();
        assert!(updated.updated.contains("51"));
        assert_eq!(
            remove_owned_profile(&updated.updated, &updated.ownership).unwrap(),
            settings
        );
    }
}
#[test]
fn unrelated_edits_survive_owned_update_and_removal() {
    let original = r#"{"profiles":{"list":[]},"theme":"before"}"#;
    let patch = plan_profile_patch(original, &owned(), None).unwrap();
    let edited = patch.updated.replace("before", "after");
    let updated = plan_profile_patch(&edited, &owned(), Some(&patch.ownership)).unwrap();
    assert_eq!(
        remove_owned_profile(&updated.updated, &updated.ownership).unwrap(),
        original.replace("before", "after")
    );
}
#[test]
fn modified_missing_duplicate_and_comment_spoofed_entries_are_not_owned() {
    let original = r#"{"profiles":{"list":[]}}"#;
    let patch = plan_profile_patch(original, &owned(), None).unwrap();
    assert!(
        remove_owned_profile(
            &patch.updated.replace("fixture.exe", "changed.exe"),
            &patch.ownership
        )
        .is_err()
    );
    assert!(plan_profile_patch(original, &owned(), Some(&patch.ownership)).is_err());
    assert!(plan_profile_patch(&patch.updated, &owned(), None).is_err());
    let comment_spoof = format!(
        "{{\"profiles\":{{\"list\":[/*{}*/]}}}}",
        patch.ownership.block
    );
    assert!(remove_owned_profile(&comment_spoof, &patch.ownership).is_err());
    let duplicate = patch
        .updated
        .replace(&patch.ownership.block, &patch.ownership.block.repeat(2));
    assert!(remove_owned_profile(&duplicate, &patch.ownership).is_err());
    assert!(plan_profile_patch(r#"{"profiles":[],"\u0070rofiles":[]}"#, &owned(), None).is_err());
}
