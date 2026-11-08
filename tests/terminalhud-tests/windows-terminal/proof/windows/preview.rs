use super::{Result, files::write_new};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};
use terminalhud_windows_terminal::{Catalog, Guid, native::fresh_guid};
pub(super) fn create(terminal: &Path) -> Result<()> {
    if !terminal.is_absolute() || !terminal.is_file() {
        return Err("explicit installed Terminal executable required".into());
    }
    let nonce = fresh_guid()?;
    let profile = fresh_guid()?;
    let task = std::env::temp_dir().join(format!("TerminalHUD-TER9-{}", nonce.bare()));
    fs::create_dir(&task)?;
    let source = Guid::parse("{c6abf713-4982-49a5-a972-000000000001}")?;
    let executable = std::env::current_exe()?;
    let command = format!(
        "\"{}\" --shell \"{}\" {}",
        executable.display(),
        task.display(),
        nonce
    );
    let synthetic = json!({"defaultProfile":source.as_str(),"profiles":{"list":[{
        "guid":source.as_str(),"name":"TER-9 isolated renderer proof","commandline":command,
        "startingDirectory":task,"closeOnExit":"always","font":{"face":"Consolas","size":16},
        "foreground":"#FFFFFF","background":"#000000","backgroundImage":null,
        "antialiasingMode":"grayscale","padding":"0","suppressApplicationTitle":true
    }]}});
    let fragment = Catalog::parse(&synthetic.to_string())?
        .preview(&source, profile.clone(), 35)?
        .utf8();
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA unavailable")?);
    let destination = local
        .join("Microsoft/Windows Terminal/Fragments/TerminalHUD")
        .join(format!("ter9-proof-{}.json", nonce.bare()));
    let record = json!({"nonce":nonce.as_str(),"profile":profile.as_str(),"terminal":terminal,"fragmentDestination":destination});
    write_new(&task.join("fragment-preview.json"), fragment.as_bytes())?;
    write_new(
        &task.join("proof-record.json"),
        serde_json::to_string_pretty(&record)?.as_bytes(),
    )?;
    println!("ProofDirectory={}", task.display());
    println!("FragmentDestination={}", destination.display());
    println!("ExternalDestinationExists={}", destination.exists());
    println!("No fragment has been installed and Terminal has not been launched.");
    Ok(())
}
