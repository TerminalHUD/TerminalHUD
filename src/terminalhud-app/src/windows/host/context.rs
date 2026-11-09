use super::UiMessage;
use super::controller::RuntimeController;
use super::hotkeys::Registrations;
use super::launch_preferences::LaunchPreferences;
use crate::{mode::Mode, windows::startup::StartupRegistration};
use std::{
    cell::{Cell, RefCell},
    io, ptr,
    sync::mpsc,
};
use terminalhud_core::config::Configuration;

use tray_icon::TrayIcon;
use windows_sys::Win32::Foundation::HWND;
pub(super) struct HostContext {
    pub(super) window: Cell<HWND>,
    pub(super) mode: Mode,
    pub(super) requires_configuration: bool,
    pub(super) launch_ready: Cell<bool>,
    pub(super) hotkey_error: Cell<u32>,
    pub(super) refresh_requested: Cell<bool>,
    pub(super) hotkeys: RefCell<Option<Registrations>>,
    pub(super) configuration: RefCell<Option<Configuration>>,
    pub(super) launch_preferences: Option<Box<dyn LaunchPreferences>>,
    pub(super) startup_registration: RefCell<Option<StartupRegistration>>,
    pub(super) configuration_error: RefCell<Option<String>>,
    pub(super) last_command_error: RefCell<Option<String>>,
    pub(super) observation_error: RefCell<Option<String>>,
    pub(super) busy: Cell<bool>,
    pub(super) deferred: Cell<bool>,
    pub(super) close_requested: Cell<bool>,
    pub(super) queue: mpsc::Receiver<UiMessage>,
    pub(super) controller: RefCell<RuntimeController>,
    pub(super) tray: RefCell<Option<TrayIcon>>,
    pub(super) error: RefCell<Option<io::Error>>,
    pub(super) sequence: Cell<u64>,
    pub(super) menu_commands: Cell<u32>,
    pub(super) quitting: Cell<bool>,
    pub(super) menu_probe: Cell<bool>,
}
impl HostContext {
    pub(super) fn new(
        queue: mpsc::Receiver<UiMessage>,
        controller: RuntimeController,
        mode: Mode,
        launch_preferences: Option<Box<dyn LaunchPreferences>>,
        requires_configuration: bool,
    ) -> Self {
        Self {
            window: Cell::new(ptr::null_mut()),
            mode,
            requires_configuration,
            launch_ready: Cell::new(false),
            hotkey_error: Cell::new(0),
            refresh_requested: Cell::new(false),
            observation_error: RefCell::new(None),
            last_command_error: RefCell::new(None),
            configuration_error: RefCell::new(None),
            hotkeys: RefCell::new(None),
            configuration: RefCell::new(None),
            launch_preferences,
            startup_registration: RefCell::new(None),
            busy: Cell::new(false),
            deferred: Cell::new(false),
            close_requested: Cell::new(false),
            queue,
            controller: RefCell::new(controller),
            tray: RefCell::new(None),
            error: RefCell::new(None),
            sequence: Cell::new(0),
            menu_commands: Cell::new(0),
            quitting: Cell::new(false),
            menu_probe: Cell::new(false),
        }
    }
}
