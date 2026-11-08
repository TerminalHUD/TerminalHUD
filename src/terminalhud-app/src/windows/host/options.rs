use std::time::Duration;
#[derive(Clone, Debug)]
pub struct HostOptions {
    pub instance: String,
    pub headless: bool,
    pub menu_probe: bool,
    pub exit_after: Option<Duration>,
}
impl Default for HostOptions {
    fn default() -> Self {
        Self {
            instance: "default".into(),
            headless: false,
            menu_probe: false,
            exit_after: None,
        }
    }
}
