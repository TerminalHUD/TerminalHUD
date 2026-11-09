use super::{NativeError, command::wide, error::last};
use std::path::Path;
use windows_sys::Win32::{
    Foundation::{FreeLibrary, HMODULE},
    System::LibraryLoader::*,
};

struct ResourceModule(HMODULE);
impl Drop for ResourceModule {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.0);
        }
    }
}
/// Read the installed settings-model resource as data, without executing DLL code
/// or asking Terminal's loader to migrate/write the user's settings.
pub fn read_inbox_defaults(terminal_executable: &Path) -> Result<String, NativeError> {
    let folder = terminal_executable
        .parent()
        .filter(|p| p.is_absolute())
        .ok_or(NativeError::Invalid("absolute Terminal path required"))?;
    let path = wide(
        folder
            .join("Microsoft.Terminal.Settings.Model.dll")
            .as_os_str(),
    )?;
    let module = unsafe {
        LoadLibraryExW(
            path.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        )
    };
    if module.is_null() {
        return Err(last("LoadLibraryExW Terminal settings data"));
    }
    let module = ResourceModule(module);
    let resource =
        unsafe { FindResourceW(module.0, 101usize as *const u16, 10usize as *const u16) };
    if resource.is_null() {
        return Err(last("FindResourceW Terminal defaults"));
    }
    let length = unsafe { SizeofResource(module.0, resource) } as usize;
    if length == 0 || length > 4 * 1024 * 1024 {
        return Err(NativeError::Invalid(
            "invalid Terminal defaults resource length",
        ));
    }
    let loaded = unsafe { LoadResource(module.0, resource) };
    if loaded.is_null() {
        return Err(last("LoadResource Terminal defaults"));
    }
    let bytes = unsafe { LockResource(loaded) };
    if bytes.is_null() {
        return Err(last("LockResource Terminal defaults"));
    }
    let bytes = unsafe { std::slice::from_raw_parts(bytes.cast::<u8>(), length) };
    String::from_utf8(bytes.to_vec())
        .map_err(|_| NativeError::Invalid("Terminal defaults are not UTF-8"))
}
