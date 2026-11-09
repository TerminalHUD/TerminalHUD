//! Initial startup can still support tray and IPC when selected shortcuts conflict.
pub(crate) fn apply_without_shortcuts(
    shortcut_error: &str,
    startup: impl FnOnce() -> Result<bool, String>,
    launch: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let changed = startup().map_err(|error| {
        format!("{shortcut_error}; startup registration commit failed: {error}; inspect startup registration")
    })?;
    launch().map_err(|error| {
        format!("{shortcut_error}; startup registration committed (changed={changed}); launch preference commit failed: {error}; inspect runtime state")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    #[test]
    fn shortcut_conflict_can_leave_launch_available() {
        let called = Cell::new(false);
        apply_without_shortcuts(
            "conflict",
            || Ok(false),
            || {
                called.set(true);
                Ok(())
            },
        )
        .unwrap();
        assert!(called.get());
    }
    #[test]
    fn startup_failure_never_commits_launch_and_partial_failure_is_reported() {
        let called = Cell::new(false);
        let error = apply_without_shortcuts(
            "conflict",
            || Err("denied".into()),
            || {
                called.set(true);
                Ok(())
            },
        )
        .unwrap_err();
        assert!(!called.get());
        assert!(error.contains("conflict") && error.contains("denied"));
        let error =
            apply_without_shortcuts("conflict", || Ok(true), || Err("launch failed".into()))
                .unwrap_err();
        assert!(error.contains("changed=true") && error.contains("launch failed"));
    }
}
