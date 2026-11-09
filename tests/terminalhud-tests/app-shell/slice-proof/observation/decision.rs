use super::Assessment;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Measurement {
    Passed,
    Failed,
    Inconclusive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fixture {
    Closed,
    Preserved,
    CleanupFailed,
}
#[derive(Debug)]
pub(crate) struct Report {
    pub measurement: Measurement,
    pub fixture: Fixture,
}
impl Report {
    /// Exit reflects the whole harness, not the cause of the measured outcome.
    pub fn exit_code(&self) -> i32 {
        match (self.measurement, self.fixture) {
            (_, Fixture::CleanupFailed) => 2,
            (Measurement::Inconclusive, _) | (_, Fixture::Preserved) => 3,
            (Measurement::Passed, Fixture::Closed) => 0,
            (Measurement::Failed, Fixture::Closed) => 2,
        }
    }
}
/// Freeze measurement before cleanup runs. Later observation/disposition cannot
/// relabel startup; callers report the independent fields even on nonzero exit.
pub(crate) fn complete(
    assessment: Option<&Assessment>,
    native_success: bool,
    cleanup: impl FnOnce(Measurement) -> Fixture,
) -> Report {
    let measurement = if assessment.is_none_or(|a| a.label() == "INCONCLUSIVE") {
        Measurement::Inconclusive
    } else if native_success {
        Measurement::Passed
    } else {
        Measurement::Failed
    };
    let fixture = cleanup(measurement);
    Report {
        measurement,
        fixture,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn clean() -> Assessment {
        Assessment {
            observed_input_activity: false,
            unexpected_foreground: false,
            insufficient_observation: false,
        }
    }
    #[test]
    fn late_activity_preserves_fixture_without_erasing_failed_startup() {
        let report = complete(Some(&clean()), false, |frozen| {
            assert_eq!(frozen, Measurement::Failed);
            Fixture::Preserved
        });
        assert_eq!(report.measurement, Measurement::Failed);
        assert_eq!(report.fixture, Fixture::Preserved);
        assert_eq!(report.exit_code(), 3);
    }
    #[test]
    fn late_activity_does_not_relabel_successful_startup_as_inconclusive() {
        let report = complete(Some(&clean()), true, |frozen| {
            assert_eq!(frozen, Measurement::Passed);
            Fixture::Preserved
        });
        assert_eq!(report.measurement, Measurement::Passed);
        assert_eq!(report.exit_code(), 3);
    }
    #[test]
    fn cleanup_failure_and_earlier_native_failure_remain_separate() {
        let report = complete(Some(&clean()), false, |_| Fixture::CleanupFailed);
        assert_eq!(report.measurement, Measurement::Failed);
        assert_eq!(report.fixture, Fixture::CleanupFailed);
        assert_eq!(report.exit_code(), 2);
    }
    #[test]
    fn cleanup_success_cannot_turn_disturbed_startup_into_a_pass() {
        let disturbed = Assessment {
            observed_input_activity: true,
            ..clean()
        };
        let report = complete(Some(&disturbed), true, |_| Fixture::Closed);
        assert_eq!(report.measurement, Measurement::Inconclusive);
        assert_eq!(report.exit_code(), 3);
    }
}
