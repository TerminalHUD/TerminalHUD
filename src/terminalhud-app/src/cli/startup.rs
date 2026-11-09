use crate::{
    protocol::Reply,
    windows::{ipc, security::Identity},
};
use std::{
    ffi::OsString,
    io::Read,
    thread,
    time::{Duration, Instant},
};
use terminalhud_core::Command;

pub(super) fn ensure_running(args: &super::Arguments) -> Result<Reply, String> {
    let instance = &args.instance;
    Identity::current()
        .and_then(|id| id.pipe_name(instance))
        .map_err(|e| e.to_string())?;
    if let Ok(reply) = ipc::call(instance, Command::Status) {
        return Ok(reply);
    }
    let host = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .with_file_name("terminalhud-shell.exe");
    let mut command: Vec<OsString> = Vec::new();
    if args.mode == crate::mode::Mode::Prototype {
        command.extend([
            OsString::from("--prototype"),
            OsString::from("--instance"),
            OsString::from(instance),
        ]);
    } else if let Some(manifest) = &args.development {
        command.push("--development".into());
        command.push(manifest.as_os_str().into());
    }
    let name = format!(
        "TerminalHUD-development-start-{}-{}.log",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    );
    let diagnostic = crate::windows::development::temporary_directory()
        .map_err(|e| e.to_string())?
        .join(name);
    let output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&diagnostic)
        .map_err(|e| e.to_string())?;
    let child = crate::windows::process_spawn::spawn(&host, &command, &output)
        .map_err(|e| e.to_string())?;
    eprintln!("TerminalHUD started_gui_pid={}", child.pid);
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Ok(reply) = ipc::call(instance, Command::Status) {
            eprintln!(
                "TerminalHUD development diagnostics: {}",
                diagnostic.display()
            );
            return Ok(reply);
        }
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let mut error = String::new();
            let _ =
                std::fs::File::open(&diagnostic).map(|f| f.take(4096).read_to_string(&mut error));
            return Err(format!(
                "GUI shell startup failed: {status}; {}",
                error.trim()
            ));
        }
        if Instant::now() >= deadline {
            if args.mode == crate::mode::Mode::Prototype {
                let _ = child.terminate_prototype();
            }
            return Err(format!(
                "GUI shell did not become ready; real owner was left alive to preserve restoration responsibility; inspect `terminalhud status` before retrying run; diagnostics={diagnostic:?}"
            ));
        }
        thread::sleep(Duration::from_millis(25));
    }
}
