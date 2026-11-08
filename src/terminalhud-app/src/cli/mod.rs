//! Console argument/exit semantics, separate from the GUI entry point.
mod arguments;
#[cfg(windows)]
mod startup;
pub use arguments::{Action, Arguments, USAGE, parse};

pub fn run(args: Vec<String>) -> Result<String, String> {
    let args = parse(args)?;
    if args.action == Action::Help {
        return Ok(USAGE.into());
    }
    #[cfg(windows)]
    {
        use crate::windows::ipc;
        let reply = match args.action {
            Action::Run => startup::ensure_running(&args.instance)?,
            Action::Command(command) => {
                ipc::call(&args.instance, command).map_err(|e| e.to_string())?
            }
            Action::Help => unreachable!(),
        };
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
