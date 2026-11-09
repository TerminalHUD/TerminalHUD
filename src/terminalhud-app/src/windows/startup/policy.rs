#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RegistryValue {
    pub kind: u32,
    pub bytes: Vec<u8>,
}
impl RegistryValue {
    pub fn command(command: &str) -> Self {
        Self {
            kind: 1,
            bytes: command
                .encode_utf16()
                .chain([0])
                .flat_map(u16::to_le_bytes)
                .collect(),
        }
    }
}
pub(super) fn prepare(
    command: &str,
    owned: &[String],
    current: Option<&RegistryValue>,
    enabled: bool,
) -> Result<Option<RegistryValue>, String> {
    if let Some(current) = current
        && !owned
            .iter()
            .any(|value| &RegistryValue::command(value) == current)
    {
        return Err(
            "HKCU Run/TerminalHUD conflicts with an unowned value; preserved unchanged".into(),
        );
    }
    Ok(enabled.then(|| RegistryValue::command(command)))
}

pub(super) fn apply(
    expected: &Option<RegistryValue>,
    desired: &Option<RegistryValue>,
    mut read: impl FnMut() -> Result<Option<RegistryValue>, String>,
    mut write: impl FnMut(&Option<RegistryValue>) -> Result<(), String>,
) -> Result<(), String> {
    if &read()? != expected {
        return Err("startup value changed since preparation; preserved unchanged".into());
    }
    if expected == desired {
        return Ok(());
    }
    write(desired).map_err(|e| format!("startup commit failed; re-read before retrying: {e}"))?;
    if &read().map_err(|e| format!("startup write succeeded but verification failed: {e}"))?
        != desired
    {
        return Err("startup write completed but value changed during verification".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
