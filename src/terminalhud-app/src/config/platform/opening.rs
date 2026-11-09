use super::apartment::Apartment;
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows_sys::Win32::UI::{
    Shell::{SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW},
    WindowsAndMessaging::SW_SHOWNORMAL,
};

pub(crate) fn open_document(path: &Path) -> Result<(), String> {
    let document = document_path(path)?;
    let _apartment = Apartment::enter()?;
    let verb = [b'o' as u16, b'p' as u16, b'e' as u16, b'n' as u16, 0];
    let mut request = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_FLAG_NO_UI | SEE_MASK_NOASYNC,
        lpVerb: verb.as_ptr(),
        lpFile: document.as_ptr(),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };
    // SAFETY: request and UTF-16 buffers outlive the synchronous shell call.
    // Exact document path; no executable lookup, shell command or parameter string.
    if unsafe { ShellExecuteExW(&mut request) } == 0 {
        let error = std::io::Error::last_os_error();
        return Err(format!(
            "cannot open configuration {} with its associated application: {error}",
            path.display()
        ));
    }
    Ok(())
}

fn document_path(path: &Path) -> Result<Vec<u16>, String> {
    if !path.is_absolute() {
        return Err("configuration document path must be absolute".into());
    }
    let mut wide: Vec<_> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err("configuration document path contains NUL".into());
    }
    wide.push(0);
    Ok(wide)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn document_path_keeps_unicode_spaces_and_metacharacters_exact() {
        let path = Path::new(r"C:\test space\配置 & $(ignored).json");
        let wide = document_path(path).unwrap();
        assert_eq!(
            String::from_utf16(&wide[..wide.len() - 1]).unwrap(),
            path.to_str().unwrap()
        );
        assert_eq!(wide.last(), Some(&0));
        assert!(document_path(Path::new("relative.json")).is_err());
        assert!(document_path(Path::new("C:\\bad\0.json")).is_err());
    }
}
