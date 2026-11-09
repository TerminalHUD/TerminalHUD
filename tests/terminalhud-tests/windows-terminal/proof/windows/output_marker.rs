//! Correlate emitted fixture bytes with the exact retained shell/session.
use serde_json::Value;
use terminalhud_windows_terminal::SessionProof;
pub(crate) fn validate(value: &Value, expected: &SessionProof) -> Result<(), &'static str> {
    if value["phase"] != "output-flushed"
        || value["nonce"].as_str() != Some(expected.nonce.as_str())
        || value["profile"].as_str() != Some(expected.profile.as_str())
        || value["session"].as_str() != Some(expected.session.as_str())
        || value["shellPid"].as_u64() != Some(u64::from(expected.shell_pid))
    {
        return Err("output-flushed marker does not match exact fixture/session");
    }
    Ok(())
}
