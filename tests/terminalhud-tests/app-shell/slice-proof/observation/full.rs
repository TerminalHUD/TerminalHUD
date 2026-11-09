//! Sticky full-scenario decisions; observation stops never become product failures.
use super::{MAX_GAP_US, Sample, Window};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Raw {
    Passed,
    Failed,
    Interrupted,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Signals {
    pub observed_input: bool,
    pub unexpected_input: bool,
    pub unexpected_foreground: bool,
    pub insufficient: bool,
    pub stimulus: bool,
}
impl Signals {
    pub fn unclear(self) -> bool {
        self.unexpected_input || self.unexpected_foreground || self.insufficient
    }
    pub fn merge(&mut self, other: Self) {
        self.observed_input |= other.observed_input;
        self.unexpected_input |= other.unexpected_input;
        self.unexpected_foreground |= other.unexpected_foreground;
        self.insufficient |= other.insufficient;
        self.stimulus |= other.stimulus;
    }
    pub fn json(self) -> serde_json::Value {
        let quality = if self.unclear() {
            "Inconclusive"
        } else if self.stimulus {
            "StimulusQualified"
        } else {
            "NoDisturbanceObserved"
        };
        serde_json::json!({
            "quality":quality,
            "observed_input_activity":self.observed_input,
            "unexpected_input_activity":self.unexpected_input,
            "unexpected_foreground":self.unexpected_foreground,
            "insufficient_observation":self.insufficient,
        })
    }
}
pub(crate) struct Control {
    pub raw: Raw,
    pub reason: Option<String>,
    pub last_completed: Option<&'static str>,
    pub stopped_phase: Option<&'static str>,
    pub signals: Signals,
    pub stopped: bool,
    tick: Option<u32>,
    previous_us: Option<u64>,
}
impl Control {
    pub fn new() -> Self {
        Self {
            raw: Raw::Interrupted,
            reason: Some("scenario not completed".into()),
            last_completed: None,
            stopped_phase: None,
            signals: Signals::default(),
            stopped: false,
            tick: None,
            previous_us: None,
        }
    }
    pub fn scan(
        &mut self,
        samples: &[Sample],
        sink: Window,
        target: Option<Window>,
        planned: Option<(u64, u64)>,
        truncated: bool,
    ) -> Signals {
        let mut seen = Signals {
            insufficient: truncated,
            stimulus: planned.is_some(),
            ..Signals::default()
        };
        for sample in samples {
            if self.previous_us.is_some_and(|t| {
                sample.elapsed_us < t || sample.elapsed_us.saturating_sub(t) > MAX_GAP_US
            }) {
                seen.insufficient = true;
            }
            if sample.input_tick.is_none() || sample.foreground.is_none() {
                seen.insufficient = true;
            }
            if let (Some(before), Some(after)) = (self.tick, sample.input_tick)
                && before != after
            {
                seen.observed_input = true;
                seen.unexpected_input |=
                    !planned.is_some_and(|(a, b)| a <= sample.elapsed_us && sample.elapsed_us <= b);
            }
            if let Some(window) = sample.foreground {
                seen.unexpected_foreground |= window != sink && Some(window) != target;
            }
            if sample.input_tick.is_some() {
                self.tick = sample.input_tick;
            }
            self.previous_us = Some(sample.elapsed_us);
        }
        self.signals.merge(seen);
        seen
    }
    pub fn gate(&mut self, phase: &'static str) -> bool {
        if !self.stopped && self.signals.unclear() {
            self.interrupt(phase, "sticky observation uncertainty");
        }
        !self.stopped
    }
    pub fn interrupt(&mut self, phase: &'static str, reason: &str) {
        if !self.stopped {
            self.raw = Raw::Interrupted;
            self.reason = Some(reason.into());
            self.stopped_phase = Some(phase);
            self.stopped = true;
        }
    }
    pub fn native_failed(&mut self, phase: &'static str, error: String) {
        if !self.stopped {
            self.raw = Raw::Failed;
            self.reason = Some(error);
            self.stopped_phase = Some(phase);
            self.stopped = true;
        }
    }
    pub fn native_result(&mut self, phase: &'static str, error: Option<String>) -> bool {
        if let Some(error) = error {
            self.native_failed(phase, error);
            return false;
        }
        self.completed(phase);
        // No downstream scripted mutation remains after the last original
        // assertion. Freeze raw completion independently of final quality.
        if phase == "quit-restore" {
            self.finish();
            true
        } else {
            self.gate(phase)
        }
    }
    pub fn completed(&mut self, phase: &'static str) {
        self.last_completed = Some(phase);
    }
    pub fn finish(&mut self) {
        if !self.stopped {
            self.raw = Raw::Passed;
            self.reason = None;
            self.stopped = true;
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FixtureBasis {
    Retained,
    NoFixture,
    Unconfirmed,
}
/// An old closed epoch cannot establish absence after any replacement attempt.
pub(crate) fn fixture_basis(
    retained: bool,
    first_closed: Option<bool>,
    relaunch_attempted: bool,
) -> FixtureBasis {
    if retained {
        FixtureBasis::Retained
    } else if first_closed == Some(true) && !relaunch_attempted {
        FixtureBasis::NoFixture
    } else {
        FixtureBasis::Unconfirmed
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::Stage;
    const SINK: Window = Window {
        hwnd: 1,
        pid: 2,
        tid: 3,
    };
    fn sample(t: u64, tick: u32) -> Sample {
        Sample {
            elapsed_us: t,
            stage: Stage::Startup,
            input_tick: Some(tick),
            foreground: Some(SINK),
        }
    }
    #[test]
    fn failed_replacement_assertion_keeps_verified_epoch_for_normal_quit() {
        let mut c = Control::new();
        c.native_result("relaunch-active", Some("native Active refused".into()));
        assert_eq!(c.raw, Raw::Failed);
        assert_eq!(
            fixture_basis(true, Some(true), true),
            FixtureBasis::Retained
        );
    }
    #[test]
    fn spawn_attempt_without_replacement_identity_never_claims_no_fixture() {
        let mut c = Control::new();
        c.native_result(
            "relaunch-active",
            Some("replacement receipt unavailable".into()),
        );
        assert_eq!(c.raw, Raw::Failed);
        assert_eq!(
            fixture_basis(false, Some(true), true),
            FixtureBasis::Unconfirmed
        );
        assert_eq!(
            fixture_basis(false, Some(true), false),
            FixtureBasis::NoFixture
        );
    }
    #[test]
    fn uncertainty_blocks_the_actual_midclose_gate_and_never_resets() {
        let mut c = Control::new();
        c.scan(&[sample(0, 4), sample(10, 5)], SINK, None, None, false);
        c.completed("startup-active");
        assert_eq!(c.last_completed, Some("startup-active"));
        assert_eq!(c.signals.json()["quality"], "Inconclusive");
        let mut close_count = 0;
        if c.gate("close-first") {
            close_count += 1;
        }
        assert_eq!(close_count, 0);
        assert_eq!(c.raw, Raw::Interrupted);
        c.scan(&[sample(20, 5)], SINK, None, None, false);
        assert!(!c.gate("relaunch-active"));
    }
    #[test]
    fn scheduled_click_does_not_erase_prior_or_hide_later_activity() {
        let mut c = Control::new();
        c.scan(
            &[sample(0, 4), sample(10, 5)],
            SINK,
            None,
            Some((0, 40)),
            false,
        );
        assert!(c.signals.observed_input);
        assert!(!c.signals.unexpected_input);
        c.scan(&[sample(20, 6)], SINK, None, None, false);
        assert!(c.signals.unexpected_input);
        c.scan(&[sample(30, 7)], SINK, None, Some((0, 40)), false);
        assert!(c.signals.unexpected_input);
    }
    #[test]
    fn null_after_intended_close_stops_relaunch_instead_of_becoming_expected() {
        let mut c = Control::new();
        let mut s = sample(10, 4);
        s.foreground = None;
        c.scan(&[sample(0, 4), s], SINK, None, None, false);
        assert!(!c.gate("relaunch-active"));
        assert_eq!(c.raw, Raw::Interrupted);
    }
    #[test]
    fn final_native_completion_survives_unclear_quality_and_late_metadata() {
        let mut c = Control::new();
        c.signals.insufficient = true;
        assert!(c.native_result("quit-restore", None));
        assert_eq!(c.raw, Raw::Passed);
        assert_eq!(c.last_completed, Some("quit-restore"));
        c.interrupt("metadata", "later error");
        assert_eq!(c.raw, Raw::Passed);
    }
    #[test]
    fn earlier_native_completion_stops_before_another_mutation_on_uncertainty() {
        let mut c = Control::new();
        c.signals.insufficient = true;
        assert!(!c.native_result("close-first", None));
        assert_eq!(c.raw, Raw::Interrupted);
        assert_eq!(c.last_completed, Some("close-first"));
    }

    #[test]
    fn later_quality_and_cleanup_cannot_erase_frozen_native_failure() {
        let mut c = Control::new();
        c.native_failed("restore-active", "native refusal".into());
        c.interrupt("quit-restore", "later unknown");
        c.finish();
        assert_eq!(c.raw, Raw::Failed);
        assert_eq!(c.stopped_phase, Some("restore-active"));
    }
}
