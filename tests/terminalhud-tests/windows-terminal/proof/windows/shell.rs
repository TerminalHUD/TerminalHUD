use super::{Result, files::write_new};
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path, time::Duration};
use terminalhud_windows_terminal::Guid;
pub(super) fn run(task: &Path, nonce: &str) -> Result<()> {
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
    print!("\x1b[2J\x1b[H\x1b[?25l\x1b[38;2;255;255;255mTerminalHUD isolated TER-9 proof\r\n");
    for row in 2..6 {
        print!("\x1b[{row};2H{}", "█".repeat(24));
    }
    std::io::stdout().flush()?;
    let mut flushed = proof;
    flushed["phase"] = json!("output-flushed");
    flushed["foregroundRgb"] = json!([255, 255, 255]);
    flushed["block"] = json!({"row":2,"rows":4,"column":2,"columns":24});
    write_new(
        &task.join("fixture-output-flushed.json"),
        flushed.to_string().as_bytes(),
    )?;
    while !task.join("exit.request").exists() {
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
