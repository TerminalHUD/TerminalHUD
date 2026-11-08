mod acquire;
mod files;
mod foreground;
mod input;
mod managed;
mod observe;
mod preview;
mod probe;
mod render;
mod session;
mod setup;
mod shell;
mod sources;
mod transitions;
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
        Some("probe") => probe::run(Path::new(
            arguments.get(2).ok_or("missing proof directory")?,
        )),
        _ => {
            Err("usage: terminal-proof preview <installed-terminal-exe> | probe <proof-dir>".into())
        }
    }
}
