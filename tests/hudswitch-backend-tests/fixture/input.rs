use super::*;

/// Inject only after OS hit-testing confirms an owned fixture at this point.
pub(crate) fn click_owned(point: POINT, expected: HWND) {
    assert_eq!(
        unsafe { WindowFromPoint(point) },
        expected,
        "refusing input outside the verified owned window"
    );
    let mut old_cursor = POINT { x: 0, y: 0 };
    assert_ne!(unsafe { GetCursorPos(&mut old_cursor) }, 0);
    let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    let mouse = |flags| INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: ((point.x - left) * 65535) / (width - 1),
                dy: ((point.y - top) * 65535) / (height - 1),
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let inputs = [
        mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK),
        mouse(MOUSEEVENTF_LEFTDOWN),
        mouse(MOUSEEVENTF_LEFTUP),
    ];
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    // Cursor restoration is a movement only; no input is sent at the old point.
    unsafe {
        SetCursorPos(old_cursor.x, old_cursor.y);
    }
    assert_eq!(
        sent,
        inputs.len() as u32,
        "SendInput refused owned-window test input"
    );
}
