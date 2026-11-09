//! Console argument/exit semantics, separate from the GUI entry point.
mod arguments;
mod configuration;
#[cfg(windows)]
pub(crate) use arguments::validate_instance;
pub use configuration::ConfigCommand;
#[cfg(windows)]
mod startup;
pub use arguments::{Action, Arguments, USAGE, parse};

pub fn run(args: Vec<String>) -> Result<String, String> {
    run_with_config(args, configuration::run)
}

fn run_with_config(
    args: Vec<String>,
    local: impl FnOnce(ConfigCommand) -> Result<String, String>,
) -> Result<String, String> {
    #[cfg(windows)]
    if args.first().is_some_and(|arg| arg == "--session-bridge") {
        crate::windows::development::run_bridge(&args[1..])?;
        return Ok(String::new());
    }
    let args = parse(args)?;
    if args.action == Action::Help {
        return Ok(USAGE.into());
    }
    if let Action::Config(command) = args.action {
        return local(command);
    }
    #[cfg(windows)]
    {
        use crate::windows::ipc;
        let reply = match args.action {
            Action::Run => startup::ensure_running(&args)?,
            Action::ReloadConfiguration => ipc::call_request(
                &args.instance,
                crate::protocol::Request::ReloadConfiguration,
            )
            .map_err(|e| e.to_string())?,
            Action::Command(command) => {
                ipc::call(&args.instance, command).map_err(|e| e.to_string())?
            }
            Action::Help | Action::Config(_) => unreachable!(),
        };
        if reply.mode != args.mode {
            return Err("runtime mode mismatch; command outcome is ambiguous".into());
        }
        if reply.code != crate::protocol::ReplyCode::Ok {
            return Err(format!(
                "command rejected: {:?}; {}",
                reply.code,
                reply.display()
            ));
        }
        Ok(reply.display())
    }
    #[cfg(not(windows))]
    {
        Err("the prototype tray/IPC host currently requires native Windows".into())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod configuration_tests;
