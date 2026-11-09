mod editor;
pub use editor::{OwnedProfileEntry, ProfilePatch, plan_profile_patch, remove_owned_profile};
mod inheritance;
pub(crate) mod parse;
mod prepare;
mod preview;
mod sources;
use crate::Guid;
pub use prepare::PreparedProfile;
pub use preview::FragmentPreview;
use serde_json::{Map, Value};
pub use sources::{GeneratedProfile, ProfileFragment};
/// Effective supplied settings; missing provider data is never invented.
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
    /// Entries without supplied identity are left unresolved, not invented.
    pub unresolved_entries: usize,
    defaults: Map<String, Value>,
}
