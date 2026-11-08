mod active;
mod files;
mod foreground_setup;
mod observe;
mod output_marker;
mod preview;
mod session;
mod setup;
mod shell;
mod sources;
mod underlay;
mod witness;
use std::path::Path;
pub(super) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run() -> Result<()> {
    let arguments: Vec<_> = std::env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("preview") => preview::create(Path::new(
            arguments
                .get(2)
                .ok_or("provide the installed Terminal executable")?,
        )),
        Some("--shell") => shell::run(
            Path::new(arguments.get(2).ok_or("missing proof directory")?),
            arguments.get(3).ok_or("missing nonce")?,
        ),
        Some("run") if arguments.len() == 3 => active::run(Path::new(&arguments[2])),
        _ => Err("usage: terminal-proof run <proof-dir> | preview <installed-terminal-exe>".into()),
    }
}
