use super::error;
use std::io;
use terminalhud_windows_terminal::Guid;
pub(in crate::windows) fn fresh() -> io::Result<Guid> {
    let mut value = windows_sys::core::GUID::from_u128(0);
    if unsafe { windows_sys::Win32::System::Com::CoCreateGuid(&mut value) } < 0 {
        return Err(error("CoCreateGuid failed"));
    }
    Guid::parse(&format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{}",
        value.data1,
        value.data2,
        value.data3,
        value.data4[0],
        value.data4[1],
        value.data4[2..]
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
    ))
    .map_err(error)
}
