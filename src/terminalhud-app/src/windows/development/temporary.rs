use super::error;
use std::{
    io,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
};
pub(crate) fn directory() -> io::Result<PathBuf> {
    let input: Vec<u16> = std::env::temp_dir()
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut output = vec![0u16; 32768];
    let length = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetLongPathNameW(
            input.as_ptr(),
            output.as_mut_ptr(),
            output.len() as u32,
        )
    } as usize;
    if length == 0 || length >= output.len() {
        return Err(error("cannot resolve long native TEMP directory"));
    }
    Ok(std::ffi::OsString::from_wide(&output[..length]).into())
}
