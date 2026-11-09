//! Recognize only the two inbox records Terminal adds to this isolated runtime.
use super::error;
use serde_json::{Value, json};
use std::io;
use terminalhud_windows_terminal::Guid;
pub(super) fn hud_profile<'a>(settings: &'a Value, identity: &Guid) -> io::Result<&'a Value> {
    let profiles = settings["profiles"]["list"]
        .as_array()
        .ok_or_else(|| error("profile list missing"))?;
    let hud = profiles
        .iter()
        .filter(|profile| profile["guid"] == identity.as_str())
        .collect::<Vec<_>>();
    if hud.len() != 1 || hud[0].get("source").is_some() || hud[0].get("updates").is_some() {
        return Err(error("expected one explicit development HUD profile"));
    }
    let inbox = [
        json!({"guid":"{61c54bbd-c2c6-5271-96e7-009a87ff44bf}","name":"Windows PowerShell","hidden":false}),
        json!({"guid":"{0caa0dad-35be-5f56-a8ff-afceeeaa6101}","name":"Command Prompt","hidden":false}),
    ];
    if profiles.len() != 1
        && (profiles.len() != 3
            || inbox.iter().any(|expected| {
                profiles
                    .iter()
                    .filter(|profile| *profile == expected)
                    .count()
                    != 1
            }))
    {
        return Err(error(
            "unknown or changed portable inbox profile; refusing fallback",
        ));
    }
    Ok(hud[0])
}
