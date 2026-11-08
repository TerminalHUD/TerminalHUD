use super::{LauncherProcess, NativeError};
use crate::LaunchPlan;
use hudswitch_core::VisibleState;

/// The measured fullscreen Passive path activates transiently. Do not spawn a
/// normal product target until a supported nonactivating path is proven.
pub(super) fn run(
    plan: &LaunchPlan,
    launch: impl FnOnce() -> Result<LauncherProcess, NativeError>,
) -> Result<LauncherProcess, NativeError> {
    if plan.intent == VisibleState::Passive {
        return Err(NativeError::Unsupported(
            "nonactivating fullscreen Windows Terminal launch is not proven; measured release activates transiently",
        ));
    }
    launch()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Guid, LaunchIdentity};
    #[test]
    fn unsupported_passive_never_calls_process_creation() {
        let guid = Guid::parse("{01234567-89ab-4cde-8fab-012345678901}").unwrap();
        let plan = LaunchPlan {
            identity: LaunchIdentity {
                nonce: guid.clone(),
                profile: guid,
            },
            intent: VisibleState::Passive,
            primary_origin: (0, 0),
        };
        let calls = std::cell::Cell::new(0);
        let result = run(&plan, || {
            calls.set(calls.get() + 1);
            Err(NativeError::Invalid("unexpected process creation"))
        });
        assert!(matches!(result, Err(NativeError::Unsupported(_))));
        assert_eq!(calls.get(), 0);
    }
}
