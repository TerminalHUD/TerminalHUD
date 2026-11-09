use crate::mode::{Mode, REAL_INSTANCE};
use std::path::PathBuf;
use terminalhud_core::Command;
pub const USAGE: &str = "Usage: terminalhud [--development MANIFEST | --prototype [--instance NAME]] run|status|interaction-toggle|visibility-toggle|passive|quit\n       terminalhud config path|validate|open|reload
       terminalhud profiles plan|apply";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Help,
    Run,
    Command(Command),
    Config(super::configuration::ConfigCommand),
    ReloadConfiguration,
    ProfilePlan,
    ProfileApply,
}
pub struct Arguments {
    pub instance: String,
    pub mode: Mode,
    pub development: Option<PathBuf>,
    pub action: Action,
}
pub fn parse(mut args: Vec<String>) -> Result<Arguments, String> {
    let mut result = Arguments {
        instance: REAL_INSTANCE.into(),
        mode: Mode::Real,
        development: None,
        action: Action::Help,
    };
    if args.as_slice() == ["--help"] {
        return Ok(result);
    }
    if args.first().is_some_and(|a| a == "profiles") {
        result.action = match args.as_slice() {
            [_, operation] if operation == "plan" => Action::ProfilePlan,
            [_, operation] if operation == "apply" => Action::ProfileApply,
            _ => return Err("expected profiles plan or profiles apply".into()),
        };
        return Ok(result);
    }
    if args.first().is_some_and(|a| a == "config") {
        result.action = if args.as_slice() == ["config", "reload"] {
            Action::ReloadConfiguration
        } else {
            Action::Config(super::configuration::parse(&args[1..])?)
        };
        return Ok(result);
    }
    if args.first().is_some_and(|a| a == "--prototype") {
        args.remove(0);
        result.mode = Mode::Prototype;
        result.instance = "default".into();
        if args.first().is_some_and(|a| a == "--instance") {
            if args.len() < 2 {
                return Err("missing instance".into());
            }
            result.instance = args.remove(1);
            args.remove(0);
        }
        validate_instance(&result.instance)?;
    } else if args.first().is_some_and(|a| a == "--development") {
        if args.len() < 2 {
            return Err("missing development manifest".into());
        }
        result.development = Some(PathBuf::from(args.remove(1)));
        args.remove(0);
    }
    if args.first().is_some_and(|a| a == "config") {
        return Err("config commands do not accept runtime mode or development flags".into());
    }
    if args.len() != 1 {
        return Err("expected exactly one command; see --help".into());
    }
    result.action = match args[0].as_str() {
        "run" => Action::Run,
        "status" => Action::Command(Command::Status),
        "interaction-toggle" => Action::Command(Command::InteractionToggle),
        "visibility-toggle" => Action::Command(Command::VisibilityToggle),
        "passive" => Action::Command(Command::Passive),
        "quit" => Action::Command(Command::Quit),
        _ => return Err("unknown command".into()),
    };
    if result.development.is_some() && result.action != Action::Run {
        return Err("--development is only used with run".into());
    }
    Ok(result)
}
pub fn validate_instance(instance: &str) -> Result<(), String> {
    if instance.is_empty()
        || instance.len() > 32
        || instance == REAL_INSTANCE
        || !instance
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid/reserved prototype instance".into());
    }
    Ok(())
}
