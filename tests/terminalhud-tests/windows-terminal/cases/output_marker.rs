#[path = "../proof/windows/output_marker.rs"]
mod marker;
use super::*;
use serde_json::json;
#[test]
fn output_marker_requires_the_exact_phase_nonce_profile_session_and_retained_shell() {
    let proof = SessionProof {
        nonce: guid(1),
        profile: guid(2),
        session: guid(3),
        shell_pid: 123,
    };
    let valid = json!({"phase":"output-flushed","nonce":proof.nonce.as_str(),"profile":proof.profile.as_str(),"session":proof.session.as_str(),"shellPid":123});
    assert!(marker::validate(&valid, &proof).is_ok());
    for key in ["nonce", "profile", "session"] {
        let mut invalid = valid.clone();
        invalid[key] = json!(guid(4).as_str());
        assert!(marker::validate(&invalid, &proof).is_err());
    }
    for (key, value) in [
        ("phase", json!("started")),
        ("shellPid", json!(124)),
        ("shellPid", json!(null)),
    ] {
        let mut invalid = valid.clone();
        invalid[key] = value;
        assert!(marker::validate(&invalid, &proof).is_err());
    }
}
