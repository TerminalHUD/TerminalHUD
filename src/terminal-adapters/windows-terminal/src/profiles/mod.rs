mod parse;
mod preview;
use crate::Guid;
pub use preview::FragmentPreview;
use serde_json::{Map, Value};
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
