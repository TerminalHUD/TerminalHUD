use serde_json::{Value, json};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use terminalhud_windows_terminal::native::fresh_guid;
use terminalhud_windows_terminal::{Catalog, Guid};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn run() -> Result<()> {
    let arguments: Vec<_> = std::env::args().collect();
    match arguments.get(1).map(String::as_str) {
        Some("preview") => preview(Path::new(
            arguments
                .get(2)
                .ok_or("provide the installed Terminal executable")?,
        )),
        Some("--shell") => shell(
            Path::new(arguments.get(2).ok_or("missing proof directory")?),
            arguments.get(3).ok_or("missing nonce")?,
        ),
        Some("probe") => Err("native probe composition is not implemented yet".into()),
        _ => {
            Err("usage: terminal-proof preview <installed-terminal-exe> | probe <proof-dir>".into())
        }
    }
}

fn write_new(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

fn preview(terminal: &Path) -> Result<()> {
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

fn shell(task: &Path, nonce: &str) -> Result<()> {
    let record: Value = serde_json::from_slice(&fs::read(task.join("proof-record.json"))?)?;
    let nonce = Guid::parse(nonce)?;
    if record["nonce"].as_str() != Some(nonce.as_str()) {
        return Err("proof nonce mismatch".into());
    }
    let profile = Guid::parse(&std::env::var("WT_PROFILE_ID")?)?;
    let session = Guid::parse(&std::env::var("WT_SESSION")?)?;
    if record["profile"].as_str() != Some(profile.as_str()) {
        return Err("Terminal started the wrong proof profile".into());
    }
    let proof = json!({"nonce":nonce.as_str(),"profile":profile.as_str(),"session":session.as_str(),"shellPid":std::process::id()});
    write_new(
        &task.join("session-proof.json"),
        proof.to_string().as_bytes(),
    )?;
    print!("\x1b[2J\x1b[H\x1b[?25l\x1b[97mTerminalHUD isolated TER-9 proof\r\n");
    for row in 10..18 {
        print!("\x1b[{row};5H{}", "█".repeat(24));
    }
    std::io::stdout().flush()?;
    while !task.join("exit.request").exists() {
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
