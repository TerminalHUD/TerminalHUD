/// Exact directory promotion used by Windows Terminal for a system WSL command.
/// Call with environment-expanded command/directory and the native system path.
#[derive(Debug, PartialEq, Eq)]
pub struct WslLaunch {
    pub commandline: String,
    /// Native Terminal clears the Win32 cwd after promoting it to --cd.
    pub starting_directory: Option<String>,
}
/// Return None when Terminal would leave the command untouched. This does not
/// execute WSL, infer a distribution, or parse/replace the selected shell.
pub fn plan_wsl_launch(
    commandline: &str,
    directory: &str,
    system_directory: &str,
) -> Option<WslLaunch> {
    if directory.is_empty() || commandline.len() < 3 {
        return None;
    }
    let terminator = commandline
        .char_indices()
        .skip(1)
        .find(|(_, c)| matches!(c, '"' | ' '))
        .map(|(i, _)| i);
    let start = usize::from(commandline.starts_with('"'));
    let executable = &commandline[start..terminator.unwrap_or(commandline.len())];
    let (parent, filename) = executable
        .rsplit_once(['\\', '/'])
        .map_or((None, executable), |(p, f)| (Some(p), f));
    if !matches!(filename, "wsl" | "wsl.exe")
        || parent.is_some_and(|p| !p.eq_ignore_ascii_case(system_directory))
    {
        return None;
    }
    let arguments = terminator.map_or("", |index| &commandline[index + 1..]);
    if arguments.contains("--cd") {
        return None;
    }
    if let Some(tilde) = arguments.find('~')
        && (tilde + 1 == arguments.len() || arguments.as_bytes()[tilde + 1] == b' ')
    {
        return None;
    }
    let directory = if directory.starts_with("//wsl$") || directory.starts_with("//wsl.localhost") {
        directory.replace('/', "\\")
    } else {
        directory.to_owned()
    };
    Some(WslLaunch {
        commandline: format!("\"{executable}\" --cd \"{directory}\" {arguments}"),
        starting_directory: None,
    })
}
