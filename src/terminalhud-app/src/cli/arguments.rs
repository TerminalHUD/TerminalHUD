use terminalhud_core::Command;

pub const USAGE: &str = "Usage: terminalhud --prototype [--instance NAME] run|status|interaction-toggle|visibility-toggle|passive|quit";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Help,
    Run,
    Command(Command),
}
pub struct Arguments {
    pub instance: String,
    pub action: Action,
}
pub fn parse(mut args: Vec<String>) -> Result<Arguments, String> {
    if args.as_slice() == ["--help"] {
        return Ok(Arguments {
            instance: "default".into(),
            action: Action::Help,
        });
    }
    if args.first().is_none_or(|a| a != "--prototype") {
        return Err("real HUD composition is not available yet; use --prototype for the explicit simulated shell".into());
    }
    args.remove(0);
    let mut instance = "default".to_string();
    if args.first().is_some_and(|a| a == "--instance") {
        if args.len() < 2 {
            return Err("missing instance".into());
        }
        instance = args.remove(1);
        args.remove(0);
    }
    if instance.is_empty()
        || instance.len() > 32
        || !instance
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid prototype instance".into());
    }
    if args.len() != 1 {
        return Err("expected exactly one prototype command; see --help".into());
    }
    let action = match args[0].as_str() {
        "run" => Action::Run,
        "status" => Action::Command(Command::Status),
        "interaction-toggle" => Action::Command(Command::InteractionToggle),
        "visibility-toggle" => Action::Command(Command::VisibilityToggle),
        "passive" => Action::Command(Command::Passive),
        "quit" => Action::Command(Command::Quit),
        _ => return Err("unknown prototype command".into()),
    };
    Ok(Arguments { instance, action })
}
