//! Installed Windows Terminal composition and owned session bridge.
mod bridge;
mod extensions;
mod launcher;
pub(in crate::windows) use launcher::InstalledLauncher;
mod preferences;
mod process;
pub(in crate::windows) use preferences::Shared;
mod session;
mod setup;
mod sources;
mod transaction;
pub(crate) use bridge::run_bridge;

/// Preview or explicitly apply the owned profile after user review.
pub(crate) fn profile_command(action: crate::cli::Action) -> Result<String, String> {
    use crate::cli::Action;
    if matches!(
        action,
        Action::ProfileRemovePlan | Action::ProfileRemovePlanDetails | Action::ProfileRemove
    ) {
        let plan = setup::prepare_removal().map_err(|error| error.to_string())?;
        if action == Action::ProfileRemove {
            plan.commit().map_err(|error| error.to_string())?;
            return Ok(
                "TerminalHUD profile removed with a backup; live sessions remain open.".into(),
            );
        }
        return Ok(plan.summary(action == Action::ProfileRemovePlanDetails));
    }
    let path = crate::config::settings_path()?;
    let config = match crate::config::validate_file(&path) {
        Ok(config) => config,
        Err(_) if matches!(std::fs::symlink_metadata(&path), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound) => {
            terminalhud_core::config::Configuration::default()
        }
        Err(error) => return Err(error),
    };
    let plan = setup::prepare(&config).map_err(|error| error.to_string())?;
    if action == Action::ProfileApply {
        plan.commit().map_err(|error| error.to_string())?;
        Ok("TerminalHUD profile applied with a backup. Windows Terminal may need to reload its settings.".into())
    } else {
        Ok(plan.summary(action == Action::ProfilePlanDetails))
    }
}
