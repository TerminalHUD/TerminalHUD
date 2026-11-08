//! Windows Terminal profile previews and isolated launch plans.
//!
//! Parsing and preview generation are read-only and portable. Native launch uses
//! explicit Windows startup intent; the application composition layer connects a
//! separately verified receipt to its chosen window backend.

use hudswitch_core::VisibleState;
use serde_json::{Map, Value, json};
use std::fmt;

#[cfg(windows)]
pub mod native;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Guid(String);
impl Guid {
    pub fn parse(value: &str) -> Result<Self, Error> {
        let bare = value
            .strip_prefix('{')
            .and_then(|v| v.strip_suffix('}'))
            .unwrap_or(value);
        if bare.len() != 36
            || !bare.bytes().enumerate().all(|(index, byte)| {
                if [8, 13, 18, 23].contains(&index) {
                    byte == b'-'
                } else {
                    byte.is_ascii_hexdigit()
                }
            })
        {
            return Err(Error::Invalid("invalid GUID"));
        }
        Ok(Self(format!("{{{}}}", bare.to_ascii_lowercase())))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn bare(&self) -> &str {
        &self.0[1..37]
    }
}
impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Settings(String),
    MissingProfile,
    NoMatchingWindow,
    UnresolvedSource,
    ElevatedProfile,
    Collision,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

/// Effective explicit settings. Dynamic-source defaults are not fabricated.
#[derive(Clone, Debug)]
pub struct Profile {
    pub guid: Guid,
    pub name: String,
    effective: Map<String, Value>,
    resolved_commandline: bool,
}
impl Profile {
    pub fn has_resolved_commandline(&self) -> bool {
        self.resolved_commandline
    }
    pub fn hidden(&self) -> bool {
        self.effective
            .get("hidden")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
    pub fn elevated(&self) -> bool {
        self.effective
            .get("elevate")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
}

pub struct Catalog {
    pub default_profile: Option<Guid>,
    pub profiles: Vec<Profile>,
    /// Entries without explicit identity are left unresolved, not invented.
    pub unresolved_entries: usize,
}

fn merge(base: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (key, value) in overlay {
        if let (Some(Value::Object(existing)), Value::Object(update)) = (base.get_mut(key), value) {
            merge(existing, update);
        } else {
            base.insert(key.clone(), value.clone());
        }
    }
}

impl Catalog {
    /// Parse supplied JSONC bytes without writing settings, resolving sources,
    /// evaluating commands, or starting applications.
    pub fn parse(text: &str) -> Result<Self, Error> {
        let settings: Value = jsonc_parser::parse_to_serde_value(
            text.trim_start_matches('\u{feff}'),
            &Default::default(),
        )
        .map_err(|error| Error::Settings(error.to_string()))?;
        let root = settings
            .as_object()
            .ok_or(Error::Invalid("settings must be an object"))?;
        let default_profile = root
            .get("defaultProfile")
            .and_then(Value::as_str)
            .and_then(|value| Guid::parse(value).ok());
        let container = root
            .get("profiles")
            .ok_or(Error::Invalid("profiles are missing"))?;
        let defaults = container
            .get("defaults")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let list = container
            .as_array()
            .or_else(|| container.get("list").and_then(Value::as_array))
            .ok_or(Error::Invalid("profile list is missing"))?;
        let mut profiles: Vec<Profile> = Vec::new();
        let mut unresolved_entries = 0;
        for entry in list {
            let Some(profile) = entry.as_object() else {
                return Err(Error::Invalid("profile must be an object"));
            };
            let Some(guid) = profile
                .get("guid")
                .and_then(Value::as_str)
                .and_then(|value| Guid::parse(value).ok())
            else {
                unresolved_entries += 1;
                continue;
            };
            if profiles.iter().any(|existing| existing.guid == guid) {
                return Err(Error::Invalid("duplicate profile GUID"));
            }
            let mut effective = defaults.clone();
            merge(&mut effective, profile);
            let name = profile
                .get("name")
                .and_then(Value::as_str)
                .ok_or(Error::Invalid("profile name is missing"))?
                .to_owned();
            let explicit_command = profile
                .get("commandline")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty());
            let dynamic = profile
                .get("source")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty());
            let resolved_commandline = explicit_command
                || (!dynamic
                    && effective
                        .get("commandline")
                        .and_then(Value::as_str)
                        .is_some_and(|value| !value.trim().is_empty()));
            profiles.push(Profile {
                guid,
                name,
                effective,
                resolved_commandline,
            });
        }
        Ok(Self {
            default_profile,
            profiles,
            unresolved_entries,
        })
    }

    pub fn default(&self) -> Result<&Profile, Error> {
        let guid = self.default_profile.as_ref().ok_or(Error::MissingProfile)?;
        self.profiles
            .iter()
            .find(|profile| &profile.guid == guid)
            .ok_or(Error::MissingProfile)
    }

    /// Preview a new HUD variant. Unsupported dynamic launch sources and elevated
    /// profiles require deliberate resolution rather than guessed commands/UAC.
    pub fn preview(
        &self,
        source: &Guid,
        new_guid: Guid,
        active_opacity: u8,
    ) -> Result<FragmentPreview, Error> {
        if active_opacity > 100 {
            return Err(Error::Invalid("opacity must be 0..100"));
        }
        if self.profiles.iter().any(|profile| profile.guid == new_guid) {
            return Err(Error::Collision);
        }
        let source = self
            .profiles
            .iter()
            .find(|profile| &profile.guid == source)
            .ok_or(Error::MissingProfile)?;
        if !source.has_resolved_commandline() {
            return Err(Error::UnresolvedSource);
        }
        if source.elevated() {
            return Err(Error::ElevatedProfile);
        }
        let mut profile = source.effective.clone();
        for key in ["guid", "name", "source", "updates", "hidden"] {
            profile.remove(key);
        }
        profile.insert("guid".into(), json!(new_guid.as_str()));
        profile.insert("name".into(), json!(format!("HUD — {}", source.name)));
        profile.insert("hidden".into(), json!(false));
        profile.insert("opacity".into(), json!(active_opacity));
        profile.insert("useAcrylic".into(), json!(false));
        let mut unfocused = match profile.remove("unfocusedAppearance") {
            Some(Value::Object(value)) => value,
            None | Some(Value::Null) => Map::new(),
            _ => {
                return Err(Error::Invalid(
                    "unfocusedAppearance must be an object or null",
                ));
            }
        };
        unfocused.insert("opacity".into(), json!(0));
        unfocused.insert("useAcrylic".into(), json!(false));
        unfocused.insert("backgroundImageOpacity".into(), json!(0));
        profile.insert("unfocusedAppearance".into(), Value::Object(unfocused));
        FragmentPreview::from_profile(profile)
    }
}

#[derive(Clone)]
pub struct FragmentPreview {
    value: Value,
}
impl FragmentPreview {
    fn from_profile(profile: Map<String, Value>) -> Result<Self, Error> {
        if profile.contains_key("updates") || profile.contains_key("source") {
            return Err(Error::Invalid("HUD fragments create independent profiles"));
        }
        Ok(Self {
            value: json!({ "profiles": [profile] }),
        })
    }
    pub fn json(&self) -> &Value {
        &self.value
    }
    pub fn utf8(&self) -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&self.value).expect("JSON values serialize")
        )
    }
}

/// A unique launch correlation. Use fresh OS-generated GUIDs for actual launches.
#[derive(Clone, Debug)]
pub struct LaunchIdentity {
    pub nonce: Guid,
    pub profile: Guid,
}
impl LaunchIdentity {
    pub fn window_name(&self) -> String {
        format!("TerminalHUD-{}", self.nonce.bare())
    }
    pub fn title(&self) -> String {
        format!("TerminalHUD {}", self.nonce.bare())
    }
}

#[derive(Clone, Debug)]
pub struct LaunchPlan {
    pub identity: LaunchIdentity,
    pub intent: VisibleState,
    pub primary_origin: (i32, i32),
}
impl LaunchPlan {
    pub fn arguments(&self) -> Vec<String> {
        vec![
            "--window".into(),
            self.identity.window_name(),
            "--fullscreen".into(),
            "--pos".into(),
            format!("{},{}", self.primary_origin.0, self.primary_origin.1),
            "new-tab".into(),
            "--profile".into(),
            self.identity.profile.to_string(),
            "--title".into(),
            self.identity.title(),
            "--suppressApplicationTitle".into(),
        ]
    }
}

/// Combine correlation evidence; title/class or launcher PID alone is insufficient.
#[derive(Clone, Debug)]
pub struct SessionProof {
    pub nonce: Guid,
    pub profile: Guid,
    pub session: Guid,
    pub shell_pid: u32,
}
#[derive(Clone, Debug)]
pub struct WindowCandidate {
    pub handle: isize,
    pub process_id: u32,
    pub class: String,
    pub title: String,
    pub existed_before_launch: bool,
}

/// `ancestors` must come from retained/live process identity checks by the native
/// caller, not untrusted shell text. Exact executable verification is also required.
pub fn correlate<'a>(
    identity: &LaunchIdentity,
    proof: &SessionProof,
    ancestors: &[u32],
    candidates: &'a [WindowCandidate],
) -> Result<&'a WindowCandidate, Error> {
    if proof.nonce != identity.nonce || proof.profile != identity.profile || proof.shell_pid == 0 {
        return Err(Error::Invalid("session handshake does not match launch"));
    }
    let mut matches = candidates.iter().filter(|candidate| {
        !candidate.existed_before_launch
            && candidate.class == "CASCADIA_HOSTING_WINDOW_CLASS"
            && candidate.title == identity.title()
            && ancestors.contains(&candidate.process_id)
    });
    let candidate = matches.next().ok_or(Error::NoMatchingWindow)?;
    if matches.next().is_some() {
        return Err(Error::Invalid("ambiguous Terminal launch identity"));
    }
    Ok(candidate)
}
