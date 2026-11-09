//! Ordinary console CLI: stdout and exit status remain synchronous.
use std::process::ExitCode;
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    if let Some(result) = terminalhud_app::cli::installed_bridge_exit(&args) {
        match result {
            Ok(code) => std::process::exit(code as i32),
            Err(error) => {
                eprintln!("terminalhud: {error}");
                std::process::exit(2);
            }
        }
    }
    match terminalhud_app::cli::run(args) {
        Ok(response) => {
            println!("{response}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("terminalhud: {error}");
            ExitCode::from(2)
        }
    }
}
