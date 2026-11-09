//! Concrete hooks around the one existing lifecycle; no second scenario engine.
mod phase;
mod report;
mod run;
use super::{Result, monitor::Monitor, target::Launched};
use crate::observation::{
    Window,
    full::{Control, Signals},
};
pub(super) use run::run;
use serde_json::{Value, json};
use terminalhud_windows_terminal::Guid;
struct Pending {
    name: &'static str,
    generation: u8,
    begin: u64,
    click: Option<(u64, u64)>,
    seen: Signals,
}
pub(super) struct Trace {
    monitor: Monitor,
    sink: Window,
    profile: Guid,
    control: Control,
    cursor: usize,
    current: Option<Window>,
    active: Option<Pending>,
    phases: Vec<Value>,
    epoch: Option<Value>,
    retained: Option<Launched>,
    restored: bool,
    relaunch_attempted: bool,
    first_absent: bool,
    first_closed: Option<bool>,
    generation: u8,
    click: Value,
    metadata_error: Option<String>,
}
impl Trace {
    pub fn relaunch_attempt(&mut self) {
        self.relaunch_attempted = true;
    }
    pub fn new(monitor: Monitor, sink: Window, profile: Guid) -> Self {
        Self {
            monitor,
            sink,
            profile,
            control: Control::new(),
            cursor: 0,
            current: None,
            active: None,
            phases: Vec::with_capacity(15),
            epoch: None,
            retained: None,
            restored: false,
            relaunch_attempted: false,
            first_absent: false,
            first_closed: None,
            generation: 1,
            click: json!({
                "attempted":false,
                "verified":false,
                "begin_us":null,
                "end_us":null,
                "concurrent_input_unexcluded":false,
            }),
            metadata_error: None,
        }
    }
}

pub(super) fn begin(
    trace: &mut Option<&mut Trace>,
    name: &'static str,
    generation: u8,
) -> Result<()> {
    if let Some(t) = trace {
        t.begin(name, generation)?;
    }
    Ok(())
}
pub(super) fn end(
    trace: &mut Option<&mut Trace>,
    result: Result<()>,
    target: Option<&Launched>,
    active: bool,
) -> Result<()> {
    if let Some(t) = trace {
        t.end(result, target, active)
    } else {
        result
    }
}
