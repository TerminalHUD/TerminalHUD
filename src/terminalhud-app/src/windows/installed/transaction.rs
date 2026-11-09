//! Guarded, journaled replacement of the user-approved settings patch.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::windows::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    ptr,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, ReplaceFileW,
};

struct Lock(PathBuf);
impl Lock {
    fn acquire(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let lock = Self(path.to_owned());
        file.sync_all()?;
        Ok(lock)
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn create(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
fn regular(path: &Path) -> io::Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(io::Error::other("expected a regular file"));
    }
    Ok(())
}
fn current(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match regular(path) {
        Ok(()) => Ok(Some(fs::read(path)?)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
fn absent(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error),
    }
}
pub(super) fn safe_parent(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing parent directory"))?;
    for directory in parent.ancestors() {
        let metadata = fs::symlink_metadata(directory)?;
        if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::other(
                "reparse point or non-directory in settings path",
            ));
        }
    }
    Ok(())
}
fn locked_settings(path: &Path) -> io::Result<(File, Vec<u8>)> {
    regular(path)?;
    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::other("settings are not a regular file"));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    Ok((file, bytes))
}
fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

pub(super) struct Plan<'a> {
    pub settings: &'a Path,
    pub expected: &'a [u8],
    pub updated: &'a [u8],
    pub staged: &'a Path,
    pub backup: &'a Path,
    pub journal: &'a Path,
    pub receipt: &'a Path,
    pub expected_receipt: Option<&'a [u8]>,
    pub new_receipt: &'a [u8],
    pub remove_receipt: bool,
}
/// Pending journal contains the new receipt and identifies the backup. A crash
/// or ambiguous replacement requires inspection; no automatic full-file rollback.
pub(super) fn apply(plan: Plan<'_>) -> io::Result<()> {
    let Plan {
        settings,
        expected,
        updated,
        staged,
        backup,
        journal,
        receipt,
        expected_receipt,
        new_receipt,
        remove_receipt,
    } = plan;
    if remove_receipt && expected_receipt.is_none() {
        return Err(io::Error::other("cannot remove an unowned receipt"));
    }
    if expected.len() > super::sources::MAX_BYTES
        || updated.len() > super::sources::MAX_BYTES
        || new_receipt.len() > super::sources::MAX_BYTES
    {
        return Err(io::Error::other(
            "settings transaction exceeds the 1 MiB limit",
        ));
    }
    if settings.parent() != staged.parent() || settings.parent() != backup.parent() {
        return Err(io::Error::other(
            "stage and backup must share the settings directory",
        ));
    }
    safe_parent(settings)?;
    safe_parent(receipt)?;
    let _lock = Lock::acquire(&receipt.with_extension("lock"))?;
    if current(receipt)?.as_deref() != expected_receipt {
        return Err(io::Error::other(
            "ownership receipt changed; review a new plan",
        ));
    }
    if !absent(backup)? || !absent(journal)? {
        return Err(io::Error::other(
            "backup or pending transaction already exists",
        ));
    }
    let (file, actual) = locked_settings(settings)?;
    if actual != expected {
        return Err(io::Error::other(
            "Terminal settings changed; review a new plan",
        ));
    }
    create(staged, updated)?;
    create(journal, new_receipt)?;
    #[cfg(test)]
    if FAIL_BEFORE_REPLACE.replace(false) {
        return Err(io::Error::other(
            "injected interruption; preserve staged file and pending journal",
        ));
    }
    // ReplaceFile requires delete sharing. Closing this deny-write/delete handle
    // creates a small race; its atomic backup records the bytes actually displaced.
    drop(file);
    #[cfg(test)]
    if RACE_AFTER_CLOSE.replace(false) {
        fs::write(settings, b"concurrent change")?;
    }
    if unsafe {
        ReplaceFileW(
            wide(settings).as_ptr(),
            wide(staged).as_ptr(),
            wide(backup).as_ptr(),
            0,
            ptr::null(),
            ptr::null(),
        )
    } == 0
    {
        return Err(io::Error::other(format!(
            "settings replacement failed: {}; preserve pending journal and staged file",
            io::Error::last_os_error()
        )));
    }
    if current(backup)?.as_deref() != Some(expected) {
        return Err(io::Error::other(
            "settings changed during replacement; preserve displaced backup and pending journal",
        ));
    }
    #[cfg(test)]
    if FAIL_AFTER_WRITE.replace(false) {
        return Err(io::Error::other(
            "injected receipt failure; preserve backup and pending journal",
        ));
    }
    let (_file, actual) = locked_settings(settings)?;
    if actual != updated || current(receipt)?.as_deref() != expected_receipt {
        return Err(io::Error::other(
            "settings or receipt changed during apply; preserve backup and pending journal",
        ));
    }
    if remove_receipt {
        fs::remove_file(receipt).map_err(|error| io::Error::other(format!("settings updated but ownership removal failed: {error}; preserve backup and pending journal")))?;
        fs::remove_file(journal).map_err(|error| io::Error::other(format!("settings updated but pending journal removal failed: {error}; preserve backup and pending journal")))
    } else {
        fs::rename(journal, receipt).map_err(|error| io::Error::other(format!("settings updated but receipt commit failed: {error}; preserve backup and pending journal")))
    }
}
#[cfg(test)]
thread_local! {
    static FAIL_AFTER_WRITE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FAIL_BEFORE_REPLACE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static RACE_AFTER_CLOSE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
#[cfg(test)]
mod tests;
