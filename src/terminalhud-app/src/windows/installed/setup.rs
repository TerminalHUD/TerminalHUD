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
    if raw.len() > 1024 * 1024 {
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
    Ok(Plan {
        sources,
        profile,
        patch,
        previous_receipt,
    })
}
impl Plan {
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
        let receipt = serde_json::json!({"schema":1,"settings":self.sources.settings,"source":self.profile.source_guid.as_str(),"ownership":self.patch.ownership.to_json(),"backup":backup});
        let bytes = serde_json::to_vec(&receipt).map_err(io::Error::other)?;
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
        })
    }
    pub fn is_owned(&self) -> bool {
        self.previous_receipt.is_some()
    }
}

#[cfg(test)]
mod tests;
