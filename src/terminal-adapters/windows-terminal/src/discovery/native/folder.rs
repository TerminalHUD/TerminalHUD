use super::super::types::*;
use super::packages::absolute_path;
use std::{path::PathBuf, ptr::null_mut};
use windows_sys::Win32::{
    System::Com::CoTaskMemFree,
    UI::Shell::{
        FOLDERID_LocalAppData, KF_FLAG_DONT_VERIFY, KF_FLAG_NO_PACKAGE_REDIRECTION,
        SHGetKnownFolderPath,
    },
};
const OP: DiscoveryOperation = DiscoveryOperation::LocalAppData;
const MAX_PATH_UNITS: usize = 32_768;
struct TaskString(*mut u16);
impl Drop for TaskString {
    fn drop(&mut self) {
        // SAFETY: SHGetKnownFolderPath task allocation or null; owned and freed exactly once even on failure.
        unsafe { CoTaskMemFree(self.0.cast()) };
    }
}
pub(super) fn local_app_data() -> Result<PathBuf, DiscoveryFailure> {
    let mut raw = TaskString(null_mut());
    // SAFETY: known ID, current-user token, writable output. Neither CREATE nor INIT is set.
    let code = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_LocalAppData,
            (KF_FLAG_DONT_VERIFY | KF_FLAG_NO_PACKAGE_REDIRECTION) as u32,
            null_mut(),
            &mut raw.0,
        )
    };
    if code < 0 {
        return Err(DiscoveryFailure::Native {
            operation: OP,
            code: code as u32,
        });
    }
    if raw.0.is_null() {
        return Err(DiscoveryFailure::Malformed {
            operation: OP,
            reason: MalformedReply::Pointer,
        });
    }
    let mut words = Vec::new();
    for index in 0..MAX_PATH_UNITS {
        // SAFETY: successful API output is an allocated NUL-terminated UTF-16 string
        // by the documented OS contract; stop at its first terminator and retain owner throughout.
        let word = unsafe { *raw.0.add(index) };
        if word == 0 {
            let path = String::from_utf16(&words).map_err(|_| DiscoveryFailure::Malformed {
                operation: OP,
                reason: MalformedReply::Utf16,
            })?;
            return absolute_path(path, OP);
        }
        words.push(word);
    }
    Err(DiscoveryFailure::Limit { operation: OP })
}
