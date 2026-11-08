//! GUI subsystem entry: no console allocation or attachment.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    if let Err(error) = run() {
        eprintln!("terminalhud-shell: {error}");
        std::process::exit(2);
    }
}
fn run() -> Result<(), String> {
    #[cfg(windows)]
    {
        terminalhud_app::windows::run_host_args(std::env::args().skip(1).collect())
    }
    #[cfg(not(windows))]
    {
        Err("prototype GUI shell requires native Windows".into())
    }
}
