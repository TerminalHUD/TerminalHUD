//! Ordinary console CLI: stdout and exit status remain synchronous.
use std::process::ExitCode;
fn main() -> ExitCode {
    match terminalhud_app::cli::run(std::env::args().skip(1).collect()) {
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
