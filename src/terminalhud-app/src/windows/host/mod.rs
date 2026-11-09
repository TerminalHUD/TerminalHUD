//! One authoritative UI thread; producers enqueue, callbacks never recurse.
mod arguments;
mod command;
mod context;
mod controller;
mod dispatch;
mod hotkey_messages;
mod hotkeys;
mod initialize;
mod menu_probe;
mod message_loop;
mod messages;
mod options;
mod queue;
mod run;
mod window;
pub use arguments::run as run_host_args;
pub use options::HostOptions;
pub use queue::{UiMessage, UiSender};
pub use run::run_host;
/// Prototype validation hook that opens the real native menu; it selects no action.
pub const MENU_PROBE_MESSAGE: u32 = windows_sys::Win32::UI::WindowsAndMessaging::WM_APP + 2;
const WM_WAKE: u32 = windows_sys::Win32::UI::WindowsAndMessaging::WM_APP + 1;
const TIMER_DURATION: usize = 1;
const TIMER_QUIT: usize = 2;
const TIMER_REFRESH: usize = 3;
