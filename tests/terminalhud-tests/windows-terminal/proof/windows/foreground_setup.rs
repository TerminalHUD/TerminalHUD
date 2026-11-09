//! Establish only the owned sink precondition; never restore focus during launch.
use super::{Result, underlay::Underlay};
use std::{
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};

pub(super) fn prepare(sink: &Underlay) -> Result<()> {
    sink.ensure_owned()?;
    let before = unsafe { GetForegroundWindow() };
    println!(
        "PHASE sink-foreground-request sink={} before={}",
        sink.hwnd() as isize,
        before as isize
    );
    let requested = unsafe { SetForegroundWindow(sink.hwnd()) } != 0;
    println!(
        "PHASE sink-foreground-result api_return={requested} observed={}",
        unsafe { GetForegroundWindow() } as isize
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        sink.ensure_owned()?;
        let observed = unsafe { GetForegroundWindow() };
        if observed == sink.hwnd() {
            println!("PHASE sink-foreground-ready sink={}", observed as isize);
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("PHASE prelaunch-refused: owned sink never became foreground; api_return={requested} sink={} observed={}; Terminal not launched", sink.hwnd() as isize, observed as isize).into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}
