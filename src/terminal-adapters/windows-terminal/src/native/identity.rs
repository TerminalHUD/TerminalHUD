use super::NativeError;
use crate::Guid;
use windows_sys::Win32::System::Com::CoCreateGuid;
pub fn fresh_guid() -> Result<Guid, NativeError> {
    let mut value = windows_sys::core::GUID::default();
    let result = unsafe { CoCreateGuid(&mut value) };
    if result < 0 {
        return Err(NativeError::Win32("CoCreateGuid", result as u32));
    }
    let tail: String = value.data4[2..]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Guid::parse(&format!(
        "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{tail}}}",
        value.data1, value.data2, value.data3, value.data4[0], value.data4[1]
    ))
    .map_err(NativeError::Correlation)
}
