//! Plan and commit a receipt-owned TerminalHUD profile in installed Terminal.
use super::{
    sources::{self, Sources},
    transaction,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use terminalhud_core::config::{Configuration, ProfileSelection};
use terminalhud_windows_terminal::{
    Catalog, GeneratedProfile, Guid, OwnedProfileEntry, PreparedProfile, ProfileFragment,
    ProfilePatch, native, plan_profile_patch, remove_owned_profile,
};

pub(super) struct Plan {
    pub sources: Sources,
    pub profile: PreparedProfile,
    pub patch: ProfilePatch,
    pub previous_receipt: Option<Vec<u8>>,
}
fn receipt_path(local: &Path) -> PathBuf {
    local.join("TerminalHUD/terminal-profile-receipt.json")
}
fn pending_path(local: &Path) -> PathBuf {
    local.join("TerminalHUD/terminal-profile-pending.json")
}
fn receipt(
    path: &Path,
    settings: &Path,
) -> io::Result<(Option<Vec<u8>>, Option<OwnedProfileEntry>)> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => {}
        Ok(_) => {
            return Err(io::Error::other(
                "profile ownership receipt is not a regular file",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok((None, None)),
        Err(error) => return Err(error),
    }
    let raw = match fs::read(path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok((None, None)),
        Err(error) => return Err(error),
    };
    if raw.len() > sources::MAX_BYTES {
        return Err(io::Error::other("profile ownership receipt exceeds limit"));
    }
    let value: serde_json::Value = serde_json::from_slice(&raw).map_err(io::Error::other)?;
    if value["schema"] != 1 || value["settings"].as_str().map(Path::new) != Some(settings) {
        return Err(io::Error::other(
            "Terminal profile ownership receipt does not match this installation",
        ));
    }
    let owned = OwnedProfileEntry::from_json(&value["ownership"]).map_err(io::Error::other)?;
    Ok((Some(raw), Some(owned)))
}
pub(super) fn prepare(config: &Configuration) -> io::Result<Plan> {
    let sources = Sources::discover()?;
    let inbox = native::read_inbox_defaults(&sources.executable).map_err(|error| {
        io::Error::other(format!(
            "cannot inspect installed Terminal defaults: {error}"
        ))
    })?;
    let generated = native::discover_generated_profiles().unwrap_or_default();
    prepare_from(config, sources, Some(&inbox), &generated)
}
fn prepare_from(
    config: &Configuration,
    sources: Sources,
    inbox: Option<&str>,
    generated: &[GeneratedProfile<'_>],
) -> io::Result<Plan> {
    match fs::symlink_metadata(pending_path(&sources.local_app_data)) {
        Ok(_) => {
            return Err(io::Error::other(
                "previous Terminal profile update needs recovery; preserve its backup and pending receipt",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let settings = sources::read(&sources.settings)?;
    let (previous_receipt, owned) =
        receipt(&receipt_path(&sources.local_app_data), &sources.settings)?;
    let clean = match &owned {
        Some(owned) => remove_owned_profile(&settings, owned).map_err(io::Error::other)?,
        None => settings.clone(),
    };
    let fragments: Vec<_> = sources
        .fragments
        .iter()
        .map(|(namespace, jsonc)| ProfileFragment { namespace, jsonc })
        .collect();
    let catalog =
        Catalog::from_sources(&clean, inbox, &fragments, generated).map_err(io::Error::other)?;
    catalog.validate_fallback().map_err(io::Error::other)?;
    let selected = match config.default_hud_profile() {
        ProfileSelection::Auto => None,
        ProfileSelection::Id(id) => Some(Guid::parse(id.as_str()).map_err(io::Error::other)?),
    };
    let guid = match &owned {
        Some(owned) => owned.guid.clone(),
        None => native::fresh_guid().map_err(io::Error::other)?,
    };
    let profile = catalog
        .prepare(selected.as_ref(), guid, config.active_background_opacity())
        .map_err(io::Error::other)?;
    let patch = plan_profile_patch(&settings, &profile.owned_profile, owned.as_ref())
        .map_err(io::Error::other)?;
    let plan = Plan {
        sources,
        profile,
        patch,
        previous_receipt,
    };
    plan.validate_size()?;
    Ok(plan)
}
impl Plan {
    fn receipt_bytes(&self, backup: &Path) -> io::Result<Vec<u8>> {
        let receipt = serde_json::json!({"schema":1,"settings":self.sources.settings,"source":self.profile.source_guid.as_str(),"ownership":self.patch.ownership.to_json(),"backup":backup});
        serde_json::to_vec(&receipt).map_err(io::Error::other)
    }
    fn validate_size(&self) -> io::Result<()> {
        if self.patch.updated.len() > sources::MAX_BYTES {
            return Err(io::Error::other(
                "proposed Terminal settings exceed the 1 MiB limit",
            ));
        }
        let parent = self
            .sources
            .settings
            .parent()
            .ok_or_else(|| io::Error::other("Terminal settings parent missing"))?;
        let backup = parent.join("settings.terminalhud-00000000-0000-0000-0000-000000000000.bak");
        if self.receipt_bytes(&backup)?.len() > sources::MAX_BYTES {
            return Err(io::Error::other(
                "proposed profile ownership receipt exceeds the 1 MiB limit",
            ));
        }
        Ok(())
    }
    pub fn commit(&self) -> io::Result<()> {
        let local = &self.sources.local_app_data;
        let directory = local.join("TerminalHUD");
        transaction::safe_parent(&directory)?;
        match fs::create_dir(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        transaction::safe_parent(&receipt_path(local))?;
        if self.patch.original == self.patch.updated && self.previous_receipt.is_some() {
            if sources::read(&self.sources.settings)? != self.patch.original
                || receipt(&receipt_path(local), &self.sources.settings)?.0 != self.previous_receipt
            {
                return Err(io::Error::other(
                    "Terminal settings or ownership changed; review a new plan",
                ));
            }
            return Ok(());
        }
        let token = native::fresh_guid()
            .map_err(io::Error::other)?
            .bare()
            .to_owned();
        let parent = self
            .sources
            .settings
            .parent()
            .ok_or_else(|| io::Error::other("Terminal settings parent missing"))?;
        let staged = parent.join(format!(".terminalhud-{token}.staged"));
        let backup = parent.join(format!("settings.terminalhud-{token}.bak"));
        let bytes = self.receipt_bytes(&backup)?;
        transaction::apply(transaction::Plan {
            settings: &self.sources.settings,
            expected: self.patch.original.as_bytes(),
            updated: self.patch.updated.as_bytes(),
            staged: &staged,
            backup: &backup,
            journal: &pending_path(local),
            receipt: &receipt_path(local),
            expected_receipt: self.previous_receipt.as_deref(),
            new_receipt: &bytes,
            remove_receipt: false,
        })
    }
    pub fn summary(&self, details: bool) -> String {
        let source = self.profile.owned_profile["name"]
            .as_str()
            .unwrap_or("unknown")
            .strip_prefix("HUD — ")
            .unwrap_or("unknown");
        let mut text = format!(
            "Windows Terminal ({:?}): {}\nSettings: {}\nSource profile: {} ({})\n{} TerminalHUD-owned profile: {}\nHUD overrides: independent profile identity/name; hidden and elevation off; source/update links removed; active opacity and acrylic off; unfocused opacity, acrylic and background image opacity zero.\nAll ordinary settings bytes are preserved. Apply stages a replacement with an exact backup beside settings and a local ownership receipt.",
            self.sources.channel,
            clean(&self.sources.executable.to_string_lossy()),
            clean(&self.sources.settings.to_string_lossy()),
            clean(source),
            self.profile.source_guid,
            if self.is_owned() { "Update" } else { "Add" },
            self.profile.hud_guid
        );
        if details {
            text.push_str("\n\nOwned entry (may contain private profile values):\n");
            text.push_str(&self.patch.ownership.block);
        }
        text
    }
    pub fn is_owned(&self) -> bool {
        self.previous_receipt.is_some()
    }
}

#[cfg(test)]
mod tests;

fn clean(value: &str) -> String {
    value.chars().filter(|c| !c.is_control() && !matches!(*c, '\u{200e}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')).take(512).collect()
}

pub(super) struct RemovalPlan {
    sources: Sources,
    original: String,
    updated: String,
    previous_receipt: Vec<u8>,
    owned: OwnedProfileEntry,
}
pub(super) fn prepare_removal() -> io::Result<RemovalPlan> {
    prepare_removal_from(Sources::base()?)
}
fn prepare_removal_from(sources: Sources) -> io::Result<RemovalPlan> {
    match fs::symlink_metadata(pending_path(&sources.local_app_data)) {
        Ok(_) => {
            return Err(io::Error::other(
                "previous Terminal profile update needs recovery; preserve its backup and pending receipt",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let (previous_receipt, owned) =
        receipt(&receipt_path(&sources.local_app_data), &sources.settings)?;
    let previous_receipt = previous_receipt
        .ok_or_else(|| io::Error::other("no TerminalHUD-owned profile receipt found"))?;
    let owned = owned.ok_or_else(|| io::Error::other("no TerminalHUD-owned profile found"))?;
    let original = sources::read(&sources.settings)?;
    let updated = remove_owned_profile(&original, &owned).map_err(io::Error::other)?;
    if updated.len() > sources::MAX_BYTES {
        return Err(io::Error::other(
            "proposed Terminal settings exceed the 1 MiB limit",
        ));
    }
    let plan = RemovalPlan {
        sources,
        original,
        updated,
        previous_receipt,
        owned,
    };
    let parent = plan
        .sources
        .settings
        .parent()
        .ok_or_else(|| io::Error::other("Terminal settings parent missing"))?;
    let backup = parent.join("settings.terminalhud-00000000-0000-0000-0000-000000000000.bak");
    if plan.journal_bytes(&backup)?.len() > sources::MAX_BYTES {
        return Err(io::Error::other(
            "proposed removal journal exceeds the 1 MiB limit",
        ));
    }
    Ok(plan)
}
impl RemovalPlan {
    fn journal_bytes(&self, backup: &Path) -> io::Result<Vec<u8>> {
        let journal = serde_json::json!({"schema":1,"operation":"remove","settings":self.sources.settings,"ownership":self.owned.to_json(),"backup":backup});
        serde_json::to_vec(&journal).map_err(io::Error::other)
    }
    pub fn summary(&self, details: bool) -> String {
        let mut text = format!(
            "Windows Terminal ({:?}): {}\nSettings: {}\nRemove TerminalHUD-owned profile: {}\nAll ordinary settings bytes are preserved. Removal stages a replacement with an exact backup beside settings and removes the matching receipt; live sessions are not closed.",
            self.sources.channel,
            clean(&self.sources.executable.to_string_lossy()),
            clean(&self.sources.settings.to_string_lossy()),
            self.owned.guid
        );
        if details {
            text.push_str("\n\nOwned entry to remove (may contain private profile values):\n");
            text.push_str(&self.owned.block);
        }
        text
    }
    pub fn commit(&self) -> io::Result<()> {
        let local = &self.sources.local_app_data;
        let token = native::fresh_guid()
            .map_err(io::Error::other)?
            .bare()
            .to_owned();
        let parent = self
            .sources
            .settings
            .parent()
            .ok_or_else(|| io::Error::other("Terminal settings parent missing"))?;
        let staged = parent.join(format!(".terminalhud-{token}.staged"));
        let backup = parent.join(format!("settings.terminalhud-{token}.bak"));
        let bytes = self.journal_bytes(&backup)?;
        transaction::apply(transaction::Plan {
            settings: &self.sources.settings,
            expected: self.original.as_bytes(),
            updated: self.updated.as_bytes(),
            staged: &staged,
            backup: &backup,
            journal: &pending_path(local),
            receipt: &receipt_path(local),
            expected_receipt: Some(&self.previous_receipt),
            new_receipt: &bytes,
            remove_receipt: true,
        })
    }
}
