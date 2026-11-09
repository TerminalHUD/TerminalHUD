//! Explicit visible proof; unattended execution runs only its registered unit tests.
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
        Ok(windows::Completion::FullProof) => (),
        Ok(windows::Completion::Startup(report)) => std::process::exit(report.exit_code()),
        Ok(windows::Completion::PreconditionRefused) => std::process::exit(3),
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
