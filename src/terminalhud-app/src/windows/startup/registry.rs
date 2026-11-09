use super::{
    StartupError,
    policy::{RegistryValue, apply as apply_checked},
};
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
    System::Registry::*,
};

const KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}
struct Handle(HKEY);
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: handle was returned by a successful registry open/create.
        unsafe { RegCloseKey(self.0) };
    }
}
pub(super) fn read() -> Result<Option<RegistryValue>, StartupError> {
    let mut raw = std::ptr::null_mut();
    // SAFETY: fixed HKCU subkey, terminated path and valid output handle.
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide(KEY).as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut raw,
        )
    };
    if result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    check(result, "read Run key")?;
    let handle = Handle(raw);
    read_handle(handle.0)
}
fn read_handle(handle: HKEY) -> Result<Option<RegistryValue>, StartupError> {
    let name = wide("TerminalHUD");
    let mut kind = 0;
    let mut length = 0;
    // SAFETY: output lengths/type are writable; null data queries length only.
    let result = unsafe {
        RegQueryValueExW(
            handle,
            name.as_ptr(),
            std::ptr::null(),
            &mut kind,
            std::ptr::null_mut(),
            &mut length,
        )
    };
    if result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    check(result, "query startup value")?;
    if length > 32768 {
        return Err("startup value is oversized; preserved unchanged"
            .to_string()
            .into());
    }
    let mut bytes = vec![0; length as usize];
    // SAFETY: buffer size matches queried size; concurrent growth returns an error.
    check(
        unsafe {
            RegQueryValueExW(
                handle,
                name.as_ptr(),
                std::ptr::null(),
                &mut kind,
                bytes.as_mut_ptr(),
                &mut length,
            )
        },
        "read startup value",
    )?;
    bytes.truncate(length as usize);
    Ok(Some(RegistryValue { kind, bytes }))
}
pub(super) fn apply(
    expected: &Option<RegistryValue>,
    desired: &Option<RegistryValue>,
) -> Result<(), StartupError> {
    apply_checked(
        expected,
        desired,
        || read().map_err(|e| e.to_string()),
        |value| write(expected, value).map_err(|e| e.to_string()),
    )
    .map_err(Into::into)
}
fn write(
    expected: &Option<RegistryValue>,
    value: &Option<RegistryValue>,
) -> Result<(), StartupError> {
    let mut raw = std::ptr::null_mut();
    let access = KEY_QUERY_VALUE | KEY_SET_VALUE;
    // SAFETY: fixed current-user key and writable output; only commit calls this.
    let result = unsafe {
        if value.is_some() {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                wide(KEY).as_ptr(),
                0,
                std::ptr::null(),
                0,
                access,
                std::ptr::null(),
                &mut raw,
                std::ptr::null_mut(),
            )
        } else {
            RegOpenKeyExW(HKEY_CURRENT_USER, wide(KEY).as_ptr(), 0, access, &mut raw)
        }
    };
    if result == ERROR_FILE_NOT_FOUND && value.is_none() {
        return if expected.is_none() {
            Ok(())
        } else {
            Err("startup key changed before write; no change applied"
                .to_string()
                .into())
        };
    }
    check(result, "open startup key for commit")?;
    let handle = Handle(raw);
    if &read_handle(handle.0)? != expected {
        return Err("startup value changed before write; preserved unchanged"
            .to_string()
            .into());
    }
    let name = wide("TerminalHUD");
    // SAFETY: live owned handle and validated value buffer; no other value is addressed.
    let result = unsafe {
        match value {
            Some(value) => RegSetValueExW(
                handle.0,
                name.as_ptr(),
                0,
                value.kind,
                value.bytes.as_ptr(),
                value.bytes.len() as u32,
            ),
            None => RegDeleteValueW(handle.0, name.as_ptr()),
        }
    };
    if result == ERROR_FILE_NOT_FOUND && value.is_none() {
        return Ok(());
    }
    check(result, "commit startup value")
}
fn check(result: u32, operation: &str) -> Result<(), StartupError> {
    if result == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("{operation}: Windows error {result}").into())
    }
}
