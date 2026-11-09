use super::document;
use super::types::{AssessmentFinding, Provenance, SuppliedSnapshot};
mod entries;
mod fragments;
mod settings;
use crate::Guid;
use serde_json::{Map, Value};

pub(super) struct Layer {
    pub values: Map<String, Value>,
    pub provenance: Provenance,
    pub location: String,
}
pub(super) struct Entry {
    pub guid: Guid,
    pub layer: Layer,
}
pub(super) struct ParsedSnapshot {
    pub defaults: Layer,
    pub user: Vec<Entry>,
    pub fragments: Vec<Entry>,
    pub disabled_sources: Vec<String>,
    pub default_profile: Option<Guid>,
}

impl ParsedSnapshot {
    pub fn parse(snapshot: SuppliedSnapshot<'_>) -> Result<Self, AssessmentFinding> {
        let settings = document::parse(snapshot.settings_jsonc, "settings")?;
        let default_profile = settings::default_profile(&settings)?;
        let disabled_sources = settings::disabled_sources(&settings)?;
        let (defaults, list) = settings::user_profiles(&settings)?;
        let user = entries::parse(list, Provenance::UserProfile, "settings.profiles", false)?;
        let fragments = fragments::parse(snapshot.terminalhud_fragments)?;
        Ok(Self {
            defaults,
            user,
            fragments,
            disabled_sources,
            default_profile,
        })
    }
}
