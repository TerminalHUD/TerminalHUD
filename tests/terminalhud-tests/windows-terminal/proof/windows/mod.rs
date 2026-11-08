mod active;
mod diagnostic;
mod files;
mod focus_evidence;
mod foreground;
mod foreground_setup;
mod launch;
mod observe;
mod preview;
mod session;
mod setup;
mod shell;
mod show;
mod sources;
mod startup;
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
        Some("diagnose") => {
            let show = show::Show::parse(arguments.get(3).map(String::as_str))?;
            let presentation = startup::Presentation::parse(show.value(), &arguments[4..])?;
            diagnostic::run(
                Path::new(arguments.get(2).ok_or("missing proof directory")?),
                show,
                presentation,
            )
        }
        _ => Err(
            "usage: terminal-proof run <proof-dir> | preview <installed-terminal-exe> | diagnose <proof-dir> <4|8> [--windowed-control]"
                .into(),
        ),
    }
}
