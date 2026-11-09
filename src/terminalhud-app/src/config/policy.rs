use super::wire::Document;
use terminalhud_core::config::ValidationError;

pub(super) fn validate(document: &Document) -> Result<(), ValidationError> {
    let guards = [
        (
            document.version == 1,
            "version",
            "only schema version 1 is supported",
        ),
        (
            document.display.monitor == "primary",
            "display.monitor",
            "MVP requires primary",
        ),
        (
            document.display.initial_presentation == "fullscreen",
            "display.initialPresentation",
            "MVP requires fullscreen",
        ),
        (
            document.display.target_count == 1,
            "display.targetCount",
            "MVP requires one target",
        ),
        (
            document.terminal.passive_background_opacity == 0,
            "terminal.passiveBackgroundOpacity",
            "MVP requires zero",
        ),
        (
            document.lifecycle.initial_state == "active",
            "lifecycle.initialState",
            "MVP creates Active",
        ),
        (
            document.lifecycle.keep_running_when_terminal_closes,
            "lifecycle.keepRunningWhenTerminalCloses",
            "MVP requires true",
        ),
        (
            document.lifecycle.leave_terminal_open_on_exit,
            "lifecycle.leaveTerminalOpenOnExit",
            "MVP requires true",
        ),
    ];
    for (valid, property, message) in guards {
        if !valid {
            return Err(ValidationError::new(property, message));
        }
    }
    Ok(())
}
