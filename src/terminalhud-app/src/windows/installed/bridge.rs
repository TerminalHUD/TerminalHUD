use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};
use terminalhud_windows_terminal::Guid;

/// The bridge refuses to start the selected shell unless Terminal confirms the
/// requested profile and session. It then survives the tray GUI for recovery.
pub(crate) fn run_bridge(args: &[String]) -> Result<(), String> {
    run(args).map_err(|error| error.to_string())
}
fn run(args: &[String]) -> io::Result<()> {
    if args.len() != 3 {
        return Err(io::Error::other(
            "installed session requires directory, nonce and profile",
        ));
    }
    let directory = Path::new(&args[0]);
    let nonce = Guid::parse(&args[1]).map_err(io::Error::other)?;
    let profile = Guid::parse(&args[2]).map_err(io::Error::other)?;
    if !directory.is_absolute() || fs::read(directory.join("nonce"))? != nonce.as_str().as_bytes() {
        return Err(io::Error::other("session nonce mismatch"));
    }
    let actual = Guid::parse(&std::env::var("WT_PROFILE_ID").map_err(io::Error::other)?)
        .map_err(io::Error::other)?;
    let session = Guid::parse(&std::env::var("WT_SESSION").map_err(io::Error::other)?)
        .map_err(io::Error::other)?;
    if profile != actual {
        return Err(io::Error::other(
            "Windows Terminal has not loaded the requested HUD profile; reload Terminal settings and retry",
        ));
    }
    let command = fs::read_to_string(directory.join("commandline"))?;
    super::super::recovery::start(nonce.bare().replace('-', ""))?;
    let directory_hint = fs::read_to_string(directory.join("starting-directory"))?;
    let child = super::process::start(&command, &directory_hint)?;
    let proof = serde_json::json!({"nonce": nonce.as_str(), "profile": profile.as_str(), "session": session.as_str(), "shellPid": std::process::id()});
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("session.tmp"))?;
    file.write_all(
        serde_json::to_string(&proof)
            .map_err(io::Error::other)?
            .as_bytes(),
    )?;
    file.sync_all()?;
    fs::rename(
        directory.join("session.tmp"),
        directory.join("session.json"),
    )?;
    child.wait()
}
