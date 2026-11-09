#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
fn main() {
    eprintln!("The native Terminal proof requires Windows.");
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows::run() {
        eprintln!("Terminal proof: {error}");
        std::process::exit(1);
    }
}
