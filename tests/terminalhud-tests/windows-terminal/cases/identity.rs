use super::*;

#[test]
fn malformed_identity_and_duplicate_profiles_are_rejected() {
    assert!(Guid::parse("not-a-guid").is_err());
    assert!(Catalog::parse(r#"{"profiles":{"list":[{"guid":"{01234567-89ab-4cde-8fab-012345678901}","name":"one"},{"guid":"{01234567-89ab-4cde-8fab-012345678901}","name":"two"}]}}"#).is_err());
}
