use super::context::HostContext;
impl HostContext {
    pub(super) fn show_probe_menu(&self) {
        if !self.menu_probe.get() || self.quitting.get() {
            return;
        }
        // Native menus use a nested message loop. No controller/queue borrow is
        // held here. Nested IPC dispatch does not mutate this tray icon object.
        if let Some(icon) = self.tray.borrow().as_ref() {
            icon.show_menu();
        }
    }
}
