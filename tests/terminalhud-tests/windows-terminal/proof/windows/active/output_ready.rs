//! Output readiness is emitted-byte evidence, never GPU/UI readiness.
use super::super::{Result, output_marker};
use std::{
    fs::File,
    io::Read,
    path::Path,
    thread,
    time::{Duration, Instant},
};
use terminalhud_windows_terminal::SessionProof;
pub(super) fn wait(task: &Path, expected: &SessionProof, alive: impl Fn() -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if !alive() {
            return Err("fixture exited before output-flushed observation".into());
        }
        match File::open(task.join("fixture-output-flushed.json")) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(4097).read_to_end(&mut bytes)?;
                if bytes.len() > 4096 {
                    return Err("oversized output-flushed marker".into());
                }
                if let Ok(value) = serde_json::from_slice(&bytes) {
                    output_marker::validate(&value, expected)?;
                    if !alive() {
                        return Err("fixture exited during output-flushed observation".into());
                    }
                    println!(
                        "ACTIVE_OUTPUT_FLUSHED {value}; emitted bytes only, not GPU readiness"
                    );
                    return Ok(());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        if Instant::now() >= deadline {
            return Err("missing or malformed output-flushed marker at deadline".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}
