//! Read-only Windows Terminal profile previews and isolated launch plans.
//! The application composition layer selects and bridges its window backend.
mod assessment;
mod error;
mod guid;
mod launch;
#[cfg(windows)]
pub mod native;
mod profiles;
pub use error::Error;
pub use guid::Guid;
pub use launch::{LaunchIdentity, LaunchPlan, SessionProof, WindowCandidate, correlate};
pub use profiles::{Catalog, FragmentPreview, Profile};

pub use assessment::{
    AssessmentFinding, ExpectedProfile, ExpectedProperty, FallbackAssessment, ProfileAssessment,
    Property, PropertyExpectation, Provenance, RefusalReason, SuppliedFragment, SuppliedSnapshot,
    UnresolvedValue, assess_snapshot,
};
