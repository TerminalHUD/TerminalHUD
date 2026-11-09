//! Explicit visible app integration proof; never part of unattended tests.
#[cfg(any(windows, test))]
mod observation;
#[cfg(windows)]
mod windows;
fn main() {
    #[cfg(windows)]
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    let startup_only = args.first().is_some_and(|s| s == "--startup-observe");
    #[cfg(windows)]
    match windows::run(args) {
        Ok(windows::Completion::FullProof | windows::Completion::StartupPass) => (),
        Ok(windows::Completion::StartupFailure) => std::process::exit(2),
        Ok(windows::Completion::Inconclusive) => std::process::exit(3),
        Err(error) => {
            if startup_only {
                eprintln!("APP_STARTUP_HARNESS_FAILED {error}; no product attribution");
            } else {
                eprintln!("APP_SLICE_FAILED {error}");
            }
            std::process::exit(2);
        }
    }
    #[cfg(not(windows))]
    eprintln!("app-slice-proof requires native Windows");
}
