//! Explicit visible app integration proof; never part of unattended tests.
#[cfg(windows)]
mod windows;
fn main() {
    #[cfg(windows)]
    if let Err(error) = windows::run(std::env::args().skip(1).collect()) {
        eprintln!("APP_SLICE_FAILED {error}");
        std::process::exit(2);
    }
    #[cfg(not(windows))]
    eprintln!("app-slice-proof requires native Windows");
}
