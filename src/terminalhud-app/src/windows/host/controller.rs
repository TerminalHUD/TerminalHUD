use super::HostOptions;
use crate::{
    mode::Mode,
    prototype::PrototypeBackend,
    windows::{
        development::{Configuration, DevelopmentLauncher},
        installed::{InstalledLauncher, Shared},
    },
};
use hudswitch_windows::WindowsBackend;
use std::io;
use terminalhud_core::{Command, CommandOutcome, ShellController, integration::Status};
pub(super) enum RuntimeController {
    Prototype(ShellController<PrototypeBackend>),
    Real(Box<ShellController<WindowsBackend<DevelopmentLauncher>>>),
    Installed(
        Box<ShellController<WindowsBackend<InstalledLauncher>>>,
        Shared,
    ),
}
impl RuntimeController {
    pub(super) fn new(options: &HostOptions) -> io::Result<Self> {
        if options.mode == Mode::Prototype {
            return Ok(Self::Prototype(ShellController::new(
                PrototypeBackend::default(),
            )));
        }
        if options.development.is_none() {
            let shared = Shared::default();
            let backend = WindowsBackend::new(InstalledLauncher::new(shared.clone()))
                .map_err(|error| io::Error::other(format!("{error:?}")))?;
            return Ok(Self::Installed(
                Box::new(ShellController::new(backend)),
                shared,
            ));
        }
        let config = Configuration::load(
            options
                .development
                .as_deref()
                .ok_or_else(|| io::Error::other("missing development manifest"))?,
        )?;
        let backend = WindowsBackend::new(DevelopmentLauncher::new(config))
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        Ok(Self::Real(Box::new(ShellController::new(backend))))
    }
    pub(super) fn launch_preferences(
        &self,
    ) -> Option<Box<dyn super::launch_preferences::LaunchPreferences>> {
        match self {
            Self::Installed(_, shared) => Some(Box::new(shared.clone())),
            _ => None,
        }
    }
    pub(super) fn status(&self) -> Status {
        match self {
            Self::Prototype(c) => c.status(),
            Self::Real(c) => c.status(),
            Self::Installed(c, _) => c.status(),
        }
    }
    pub(super) fn refresh(&mut self) -> Result<(), String> {
        match self {
            Self::Prototype(_) => Ok(()),
            Self::Real(c) => c.refresh().map(|_| ()).map_err(|e| format!("{e:?}")),
            Self::Installed(c, _) => c.refresh().map(|_| ()).map_err(|e| format!("{e:?}")),
        }
    }
    pub(super) fn execute(&mut self, command: Command) -> Result<CommandOutcome<String>, String> {
        match self {
            Self::Prototype(c) => c
                .execute(command)
                .map(|o| CommandOutcome {
                    status: o.status,
                    focus_restore_error: o.focus_restore_error.map(|e| e.to_string()),
                    should_exit: o.should_exit,
                })
                .map_err(|e| format!("{e:?}")),
            Self::Real(c) => c
                .execute(command)
                .map(|o| CommandOutcome {
                    status: o.status,
                    focus_restore_error: o.focus_restore_error.map(|e| format!("{e:?}")),
                    should_exit: o.should_exit,
                })
                .map_err(|e| format!("{e:?}")),
            Self::Installed(c, _) => c
                .execute(command)
                .map(|o| CommandOutcome {
                    status: o.status,
                    focus_restore_error: o.focus_restore_error.map(|e| format!("{e:?}")),
                    should_exit: o.should_exit,
                })
                .map_err(|e| format!("{e:?}")),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_real_configuration_never_falls_back_to_simulation() {
        let options = HostOptions {
            mode: Mode::Real,
            development: Some("relative-invalid-runtime.json".into()),
            ..HostOptions::default()
        };
        assert!(RuntimeController::new(&options).is_err());
        let mut prototype = RuntimeController::new(&HostOptions::default()).unwrap();
        assert_eq!(
            prototype
                .execute(Command::VisibilityToggle)
                .unwrap()
                .status
                .state,
            Some(terminalhud_core::integration::State::Active)
        );
    }
}
