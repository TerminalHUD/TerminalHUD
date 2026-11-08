//! One guarded physical click, routed only to the owned Passive underlay.
use super::{
    super::{Result, observe, underlay::Underlay},
    identity, input_recovery,
    launch::Launched,
};
use windows_sys::Win32::{
    Foundation::*,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
pub(super) fn click_sink(target: &Launched, sink: &Underlay) -> Result<()> {
    require_neutral_buttons()?;
    let rect = identity::fullscreen(target)?;
    let point = POINT {
        x: (rect.left + rect.right) / 2,
        y: (rect.top + rect.bottom) / 2,
    };
    identity::owned_point(target, sink, point)?;
    if unsafe { WindowFromPoint(point) } != sink.hwnd() {
        return Err("refusing physical input: hit target is not the owned sink".into());
    }
    let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    if width <= 1 || height <= 1 {
        return Err("invalid virtual desktop dimensions".into());
    }
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
    let input = [
        mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK),
        mouse(MOUSEEVENTF_LEFTDOWN),
        mouse(MOUSEEVENTF_LEFTUP),
    ];
    sink.ensure_owned()?;
    identity::ensure(target)?;
    if unsafe { WindowFromPoint(point) } != sink.hwnd() {
        return Err("owned click hit test changed before injection".into());
    }
    if unsafe { GetForegroundWindow() } != sink.hwnd() {
        return Err("owned click lost sink foreground before injection".into());
    }
    require_neutral_buttons()?;
    let before = sink.clicks();
    let sent = unsafe {
        SendInput(
            input.len() as u32,
            input.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    input_recovery::complete(sent, || unsafe {
        SendInput(
            1,
            &mouse(MOUSEEVENTF_LEFTUP),
            std::mem::size_of::<INPUT>() as i32,
        ) == 1
    })?;
    observe::wait(|| sink.clicks() == before + 1)?;
    println!("ACTIVE_PROOF physical Passive click reached owned sink exactly once");
    Ok(())
}

fn require_neutral_buttons() -> Result<()> {
    let states = [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2]
        .map(|key| unsafe { GetAsyncKeyState(i32::from(key)) });
    if !input_recovery::neutral(states) {
        return Err("owned click requires all mouse buttons released".into());
    }
    Ok(())
}
