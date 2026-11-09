use std::{collections::HashSet, path::PathBuf};
use terminalhud_app::{mode::REAL_INSTANCE, windows::ipc};
use terminalhud_core::Command;
use terminalhud_windows_terminal::{Guid, native};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
mod active;
mod client;
#[path = "../../../windows-terminal/proof/windows/foreground_setup.rs"]
mod foreground_setup;
mod lifecycle;
#[path = "../../../windows-terminal/proof/windows/observe.rs"]
mod observe;
mod startup;
mod startup_identity;
mod target;
#[path = "../../../windows-terminal/proof/windows/underlay/mod.rs"]
mod underlay;
#[path = "../../../windows-terminal/proof/windows/witness.rs"]
mod witness;
pub(super) fn run(args: Vec<String>) -> Result<()> {
    if args.len() != 1 {
        return Err("explicit absolute development manifest required".into());
    }
    let manifest = PathBuf::from(&args[0]);
    if !manifest.is_absolute() {
        return Err("absolute manifest required".into());
    }
    client::require_no_owner()?;
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&manifest)?)?;
    let terminal = PathBuf::from(
        value["terminal"]
            .as_str()
            .ok_or("manifest terminal missing")?,
    );
    let profile = Guid::parse(
        value["profile"]
            .as_str()
            .ok_or("manifest profile missing")?,
    )?;
    let witnesses = witness::Witness::capture()?;
    let before: HashSet<_> = native::windows()?.iter().map(|w| w.handle).collect();
    let sink = underlay::Underlay::start()?;
    let result = (|| {
        foreground_setup::prepare(&sink)?;
        let owner = client::Owner::start(&manifest)?;
        let result = lifecycle::demonstrate(&owner, &terminal, &profile, &before, &sink);
        // The guard issues only authenticated Quit to this exact retained owner;
        // it never kills a process or closes a terminal after restoration.
        drop(owner);
        result
    })();
    drop(sink);
    let witness_result = witnesses.check();
    if let Err(error) = &witness_result {
        eprintln!("APP_WITNESS_CHECK_FAILED {error}");
    }
    result?;
    witness_result?;
    println!(
        "APP_SLICE_PASS state/lifecycle/restoration with live bridge/session; no keyboard input injected. Actual post-Quit interactive command and physical selected shortcuts remain untested."
    );
    Ok(())
}
