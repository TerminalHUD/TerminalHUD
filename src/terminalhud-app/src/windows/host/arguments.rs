use crate::{
    cli::validate_instance,
    mode::{Mode, REAL_INSTANCE},
};
use std::time::Duration;
pub fn run(args: Vec<String>) -> Result<(), String> {
    let mut options = super::HostOptions::default();
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("--prototype") => {}
        Some("--development") => {
            options.mode = Mode::Real;
            options.instance = REAL_INSTANCE.into();
            options.development = Some(args.next().ok_or("missing development manifest")?.into());
        }
        None => {
            options.mode = Mode::Real;
            options.instance = REAL_INSTANCE.into();
        }
        _ => return Err("unknown host argument".into()),
    }
    while let Some(arg) = args.next() {
        if options.mode == Mode::Real {
            return Err("test hooks and instance overrides require prototype mode".into());
        }
        match arg.as_str() {
            "--instance" => options.instance = args.next().ok_or("missing instance")?,
            "--headless" => options.headless = true,
            "--menu-probe" => options.menu_probe = true,
            "--exit-after-ms" => {
                let value: u64 = args
                    .next()
                    .ok_or("missing duration")?
                    .parse()
                    .map_err(|_| "invalid duration")?;
                if !(100..=60_000).contains(&value) {
                    return Err("duration must be 100–60000 ms".into());
                }
                options.exit_after = Some(Duration::from_millis(value));
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
    }
    if options.mode == Mode::Prototype {
        validate_instance(&options.instance)?;
    }
    super::run_host(options).map_err(|e| e.to_string())
}
