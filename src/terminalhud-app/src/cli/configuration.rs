use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigCommand {
    Path,
    Validate,
    Open,
}

pub(super) fn parse(args: &[String]) -> Result<ConfigCommand, String> {
    match args {
        [command] => match command.as_str() {
            "path" => Ok(ConfigCommand::Path),
            "validate" => Ok(ConfigCommand::Validate),
            "open" => Ok(ConfigCommand::Open),
            _ => Err("expected config path, validate or open".into()),
        },
        _ => Err("expected exactly one config command: path, validate or open".into()),
    }
}

pub(super) fn run(command: ConfigCommand) -> Result<String, String> {
    let path = crate::config::settings_path()?;
    run_at(command, &path, crate::config::open_document)
}

pub(super) fn run_at(
    command: ConfigCommand,
    path: &Path,
    opener: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<String, String> {
    match command {
        ConfigCommand::Path => Ok(path.display().to_string()),
        ConfigCommand::Validate => {
            crate::config::validate_file(path)?;
            Ok(format!("Valid configuration: {}", path.display()))
        }
        ConfigCommand::Open => {
            crate::config::ensure_file(path)?;
            opener(path)?;
            Ok(format!("Opened configuration: {}", path.display()))
        }
    }
}
