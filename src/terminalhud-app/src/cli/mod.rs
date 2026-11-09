//! Console argument/exit semantics, separate from the GUI entry point.
mod arguments;
#[cfg(windows)]
pub(crate) use arguments::validate_instance;
#[cfg(windows)]
mod startup;
pub use arguments::{Action, Arguments, USAGE, parse};

pub fn run(args: Vec<String>) -> Result<String, String> {
    #[cfg(windows)]
    if args.first().is_some_and(|arg| arg == "--session-bridge") {
        crate::windows::development::run_bridge(&args[1..])?;
        return Ok(String::new());
    }
    let args = parse(args)?;
    if args.action == Action::Help {
        return Ok(USAGE.into());
    }
    #[cfg(windows)]
    {
        use crate::windows::ipc;
        let reply = match args.action {
            Action::Run => startup::ensure_running(&args)?,
            Action::Command(command) => {
                ipc::call(&args.instance, command).map_err(|e| e.to_string())?
            }
            Action::Help => unreachable!(),
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
