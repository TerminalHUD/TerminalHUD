//! Explicit, bounded native/Tao comparison; not an unattended test.
#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
#[path = "loop_probe.rs"]
mod probe;
#[cfg(windows)]
fn main() {
    let mode = std::env::args().nth(1).expect("native|tao");
    let report = std::env::args().nth(2).expect("task-owned report path");
    let started = std::time::Instant::now();
    let (events, icon_rect) = match mode.as_str() {
        "native" => probe::native(),
        "tao" => probe::tao(),
        _ => panic!("native|tao"),
    };
    assert_eq!(events, 1, "queued producer event did not reach loop");
    std::fs::write(
        report,
        format!(
            "mode={mode} elapsed_ms={} queued_events={events} icon_rect={icon_rect} console={}\n",
            started.elapsed().as_millis(),
            unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() } as usize
        ),
    )
    .unwrap();
}
#[cfg(not(windows))]
fn main() {
    eprintln!("native desktop comparison requires Windows");
}
