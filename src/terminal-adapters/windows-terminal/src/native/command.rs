use super::NativeError;
use std::os::windows::ffi::OsStrExt;
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
    plan: &crate::LaunchPlan,
    executable: &std::path::Path,
    command: &[String],
    reload_environment: Option<bool>,
) -> Result<Vec<u16>, NativeError> {
    if command.iter().any(|argument| argument.contains(';')) {
        return Err(NativeError::Invalid(
            "Terminal command separators are forbidden",
        ));
    }
    let terminal_arguments = reload_environment.map_or_else(
        || plan.arguments(),
        |reload| plan.arguments_with_environment(reload),
    );
    let arguments = std::iter::once(executable.to_string_lossy().into_owned())
        .chain(terminal_arguments)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn environment_flag_reaches_native_command_before_bridge() {
        let plan = crate::LaunchPlan {
            identity: crate::LaunchIdentity {
                nonce: crate::Guid::parse("{01234567-89ab-4cde-8fab-012345678901}").unwrap(),
                profile: crate::Guid::parse("{01234567-89ab-4cde-8fab-012345678902}").unwrap(),
            },
            intent: hudswitch_core::VisibleState::Active,
            primary_origin: (0, 0),
        };
        let executable = std::path::Path::new(r"C:\fixture\WindowsTerminal.exe");
        for (reload, expected) in [
            (true, "--reloadEnvironment"),
            (false, "--inheritEnvironment"),
        ] {
            let encoded = arguments(
                &plan,
                executable,
                &["fixture-bridge.exe".into()],
                Some(reload),
            )
            .unwrap();
            let command = String::from_utf16(&encoded[..encoded.len() - 1]).unwrap();
            assert!(command.ends_with(&format!("\"{expected}\" \"fixture-bridge.exe\"")));
        }
    }
}
