use super::apartment::Apartment;
use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};
use windows_sys::Win32::{
    System::Com::CoTaskMemFree,
    UI::Shell::{FOLDERID_LocalAppData, KF_FLAG_DONT_VERIFY, SHGetKnownFolderPath},
};

pub(crate) fn settings_path() -> Result<PathBuf, String> {
    let _apartment = Apartment::enter()?;
    let mut raw = std::ptr::null_mut();
    // SAFETY: known folder ID and writable output pointer; no create flag is used.
    let result = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_LocalAppData,
            KF_FLAG_DONT_VERIFY as u32,
            std::ptr::null_mut(),
            &mut raw,
        )
    };
    let folder = if result >= 0 && !raw.is_null() {
        // SAFETY: successful API output is a null-terminated allocated UTF-16 string.
        let value = unsafe {
            let mut length = 0;
            while *raw.add(length) != 0 {
                length += 1;
            }
            OsString::from_wide(std::slice::from_raw_parts(raw, length))
        };
        Some(PathBuf::from(value))
    } else {
        None
    };
    // SAFETY: output is task-allocated (or null), even on failure; freed exactly once.
    unsafe { CoTaskMemFree(raw.cast()) };
    let folder =
        folder.ok_or_else(|| format!("cannot resolve Local App Data: HRESULT {result:#010x}"))?;
    if !folder.is_absolute() {
        return Err("Local App Data is not an absolute path".into());
    }
    Ok(folder.join("TerminalHUD").join("settings.json"))
}
