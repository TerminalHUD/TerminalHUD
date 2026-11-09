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
    if args
        .first()
        .is_some_and(|arg| arg == "--installed-session-bridge")
    {
        crate::windows::installed::run_bridge(&args[1..])?;
        return Ok(String::new());
    }
    #[cfg(windows)]
    if args.first().is_some_and(|arg| arg == "--session-bridge") {
        crate::windows::development::run_bridge(&args[1..])?;
        return Ok(String::new());
    }
    let args = parse(args)?;
    if args.action == Action::Help {
        return Ok(USAGE.into());
    }
    if matches!(args.action, Action::ProfilePlan | Action::ProfileApply) {
        #[cfg(windows)]
        {
            return crate::windows::installed::profile_command(args.action == Action::ProfileApply);
        }
        #[cfg(not(windows))]
        {
            return Err("installed profiles require native Windows".into());
        }
    }
    if let Action::Config(command) = args.action {
        return local(command);
    }
    #[cfg(windows)]
    {
        use crate::windows::ipc;
        let reply = runtime_reply(&args, startup::ensure_running, |instance, request| {
            ipc::call_request(instance, request).map_err(|e| e.to_string())
        })?;
        if reply.mode != args.mode {
            return Err("runtime mode mismatch; command outcome is ambiguous".into());
        }
        if !matches!(
            reply.code,
            crate::protocol::ReplyCode::Ok | crate::protocol::ReplyCode::RequiresRelaunch
        ) {
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

#[cfg(any(windows, test))]
fn runtime_reply(
    args: &Arguments,
    start: impl FnOnce(&Arguments) -> Result<crate::protocol::Reply, String>,
    send: impl FnOnce(&str, crate::protocol::Request) -> Result<crate::protocol::Reply, String>,
) -> Result<crate::protocol::Reply, String> {
    use crate::protocol::Request;
    match args.action {
        Action::Run => start(args),
        Action::ReloadConfiguration => send(&args.instance, Request::ReloadConfiguration),
        Action::Command(command) => send(&args.instance, Request::Command(command)),
        Action::Help | Action::Config(_) | Action::ProfilePlan | Action::ProfileApply => {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod configuration_tests;
