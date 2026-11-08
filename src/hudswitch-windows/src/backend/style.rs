use super::*;

pub(super) fn exstyle(hwnd: HWND) -> Result<u32, Error> {
    unsafe {
        SetLastError(0);
    }
    let value = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    if value == 0 && unsafe { GetLastError() } != 0 {
        Err(last_error("GetWindowLongPtrW"))
    } else {
        Ok(value as u32)
    }
}

pub(super) fn set_style(target: &Target, mask: u32, bits: u32) -> Result<(), Error> {
    target.ensure()?;
    let previous = exstyle(target.window.hwnd)?;
    let desired = (previous & !mask) | (bits & mask);
    let changed = (previous ^ desired) & mask;
    if changed == 0 {
        return Ok(());
    }
    // Record attempted ownership before the API, including partial/error cases.
    target
        .changed_style
        .set(target.changed_style.get() | changed);
    unsafe {
        SetLastError(0);
    }
    let value = unsafe { SetWindowLongPtrW(target.window.hwnd, GWL_EXSTYLE, desired as isize) };
    if value == 0 && unsafe { GetLastError() } != 0 {
        Err(last_error("SetWindowLongPtrW"))
    } else {
        Ok(())
    }
}
