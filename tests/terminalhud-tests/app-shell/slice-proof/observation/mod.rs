//! Bounded observations describe disturbance; they cannot prove absence of input.
mod classify;
pub(crate) mod decision;
pub(crate) mod full;
pub(crate) use classify::classify;
#[cfg(test)]
mod tests;

pub(crate) const MAX_GAP_US: u64 = 100_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Window {
    pub hwnd: isize,
    pub pid: u32,
    pub tid: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    Setup,
    Startup,
    InspectActive,
    InspectRecovered,
    Quit,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sample {
    pub elapsed_us: u64,
    pub stage: Stage,
    /// Raw session-specific dwTime; equality changes only, never ordered/subtracted.
    pub input_tick: Option<u32>,
    /// None includes NULL, failed ownership query or unstable double-read HWND.
    pub foreground: Option<Window>,
}
#[derive(Clone, Copy)]
pub(crate) struct Span {
    pub start_us: u64,
    pub end_us: u64,
    /// Startup classification excludes later Quit/cleanup activity.
    pub through_quit: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Assessment {
    pub observed_input_activity: bool,
    pub unexpected_foreground: bool,
    pub insufficient_observation: bool,
}
impl Assessment {
    pub fn label(&self) -> &'static str {
        if self.observed_input_activity
            || self.unexpected_foreground
            || self.insufficient_observation
        {
            "INCONCLUSIVE"
        } else {
            "NO_DISTURBANCE_OBSERVED"
        }
    }
}
