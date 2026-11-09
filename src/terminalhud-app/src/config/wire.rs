use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Document {
    pub version: u64,
    #[serde(default)]
    pub hotkeys: Hotkeys,
    #[serde(default)]
    pub startup: Startup,
    #[serde(default)]
    pub display: Display,
    #[serde(default)]
    pub terminal: Terminal,
    #[serde(default)]
    pub lifecycle: Lifecycle,
}

macro_rules! section {
    ($name:ident { $($field:ident: $ty:ty = $value:expr),* $(,)? }) => {
        #[derive(Deserialize, Serialize)]
        #[serde(default, rename_all = "camelCase", deny_unknown_fields)]
        pub(super) struct $name { $(pub $field: $ty),* }
        impl Default for $name {
            fn default() -> Self { Self { $($field: $value),* } }
        }
    };
}

section!(Hotkeys {
    toggle_interaction: String = "Win+Backtick".into(),
    toggle_visibility: String = "Win+Shift+Backtick".into(),
});
section!(Startup {
    start_with_windows: bool = true
});
section!(Display {
    monitor: String = "primary".into(),
    initial_presentation: String = "fullscreen".into(),
    target_count: u64 = 1,
});
section!(Terminal {
    active_background_opacity: i64 = 35,
    passive_background_opacity: i64 = 0,
    default_hud_profile: String = "auto".into(),
});
section!(Lifecycle {
    initial_state: String = "active".into(),
    keep_running_when_terminal_closes: bool = true,
    leave_terminal_open_on_exit: bool = true,
});
