use super::UiMessage;
use crate::prototype::PrototypeBackend;
use std::{
    cell::{Cell, RefCell},
    io, ptr,
    sync::mpsc,
};
use terminalhud_core::ShellController;
use tray_icon::TrayIcon;
use windows_sys::Win32::Foundation::HWND;
pub(super) struct HostContext {
    pub(super) window: Cell<HWND>,
    pub(super) busy: Cell<bool>,
    pub(super) deferred: Cell<bool>,
    pub(super) close_requested: Cell<bool>,
    pub(super) queue: mpsc::Receiver<UiMessage>,
    pub(super) controller: RefCell<ShellController<PrototypeBackend>>,
    pub(super) tray: RefCell<Option<TrayIcon>>,
    pub(super) error: RefCell<Option<io::Error>>,
    pub(super) sequence: Cell<u64>,
    pub(super) menu_commands: Cell<u32>,
    pub(super) quitting: Cell<bool>,
    pub(super) menu_probe: Cell<bool>,
}
impl HostContext {
    pub(super) fn new(queue: mpsc::Receiver<UiMessage>) -> Self {
        Self {
            window: Cell::new(ptr::null_mut()),
            busy: Cell::new(false),
            deferred: Cell::new(false),
            close_requested: Cell::new(false),
            queue,
            controller: RefCell::new(ShellController::new(PrototypeBackend::default())),
            tray: RefCell::new(None),
            error: RefCell::new(None),
            sequence: Cell::new(0),
            menu_commands: Cell::new(0),
            quitting: Cell::new(false),
            menu_probe: Cell::new(false),
        }
    }
}
