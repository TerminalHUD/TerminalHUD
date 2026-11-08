use std::os::windows::ffi::OsStrExt;
use terminalhud_windows_terminal::native::NativeError;
pub(super) fn wide(value: &std::ffi::OsStr) -> Result<Vec<u16>, NativeError> {
    let mut value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err(NativeError::Invalid("NUL in native path/argument"));
    }
    value.push(0);
    Ok(value)
}

/// Standard Windows argv quoting; this does not invoke a command shell.
fn quote(value: &str) -> String {
    let mut output = String::from("\"");
    let mut slashes = 0;
    for character in value.chars() {
        if character == '\\' {
            slashes += 1;
            continue;
        }
        if character == '"' {
            output.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
        } else {
            output.extend(std::iter::repeat_n('\\', slashes));
        }
        output.push(character);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n('\\', slashes * 2));
    output.push('"');
    output
}

pub(super) fn arguments(
    plan: &terminalhud_windows_terminal::LaunchPlan,
    executable: &std::path::Path,
    command: &[String],
    presentation: super::Presentation,
) -> Result<Vec<u16>, NativeError> {
    if command.iter().any(|argument| argument.contains(';')) {
        return Err(NativeError::Invalid(
            "Terminal command separators are forbidden",
        ));
    }
    let arguments = std::iter::once(executable.to_string_lossy().into_owned())
        .chain(presentation.arguments(plan).map_err(NativeError::Invalid)?)
        .chain(command.iter().cloned())
        .map(|argument| quote(&argument))
        .collect::<Vec<_>>()
        .join(" ");
    let result = wide(std::ffi::OsStr::new(&arguments))?;
    if result.len() > 32767 {
        return Err(NativeError::Invalid("command line too long"));
    }
    Ok(result)
}
