use super::*;

pub(super) fn position(
    target: &Target,
    topmost: Option<bool>,
    visible: Option<bool>,
) -> Result<(), Error> {
    target.ensure()?;
    let before = exstyle(target.window.hwnd)?;
    let was_visible = unsafe { IsWindowVisible(target.window.hwnd) } != 0;
    let mut flags =
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_ASYNCWINDOWPOS;
    let after = match topmost {
        Some(value) => {
            if value != (before & WS_EX_TOPMOST != 0) {
                target.changed_topmost.set(true);
            }
            if value { HWND_TOPMOST } else { HWND_NOTOPMOST }
        }
        None => {
            flags |= SWP_NOZORDER;
            null_mut()
        }
    };
    if let Some(value) = visible
        && value != was_visible
    {
        target.changed_visibility.set(true);
    }
    if unsafe { SetWindowPos(target.window.hwnd, after, 0, 0, 0, 0, flags) } == 0 {
        return Err(last_error("SetWindowPos"));
    }
    if let Some(value) = visible {
        target.ensure()?;
        if unsafe {
            ShowWindowAsync(
                target.window.hwnd,
                if value { SW_SHOWNOACTIVATE } else { SW_HIDE },
            )
        } == 0
        {
            return Err(last_error("ShowWindowAsync"));
        }
    }
    wait_until(target, "window presentation", || {
        let style = exstyle(target.window.hwnd)?;
        Ok(
            topmost.is_none_or(|value| value == (style & WS_EX_TOPMOST != 0))
                && visible.is_none_or(|value| {
                    value == (unsafe { IsWindowVisible(target.window.hwnd) } != 0)
                }),
        )
    })
}
