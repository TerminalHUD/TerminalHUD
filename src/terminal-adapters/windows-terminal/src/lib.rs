//! Read-only Windows Terminal profile previews and isolated launch plans.
//! The application composition layer selects and bridges its window backend.
mod assessment;
mod discovery;
mod error;
mod guid;
mod launch;
#[cfg(windows)]
pub mod native;
mod profiles;
pub use error::Error;
pub use guid::Guid;
pub use launch::{LaunchIdentity, LaunchPlan, SessionProof, WindowCandidate, correlate};
pub use profiles::{
    Catalog, FragmentPreview, GeneratedProfile, OwnedProfileEntry, PreparedProfile, Profile,
    ProfileFragment, ProfilePatch, WslLaunch, plan_profile_patch, plan_wsl_launch,
    remove_owned_profile,
};

pub use assessment::{
    AssessmentFinding, ExpectedProfile, ExpectedProperty, FallbackAssessment, ProfileAssessment,
    Property, PropertyExpectation, Provenance, RefusalReason, SuppliedFragment, SuppliedSnapshot,
    UnresolvedValue, assess_snapshot,
};

#[cfg(windows)]
pub use discovery::discover_registered_terminals;
pub use discovery::{
    DiscoveryFailure, DiscoveryOperation, MalformedReply, PackageArchitecture, RegisteredChannel,
    RegisteredPackageCandidate, RegisteredPackageIdentity, RegisteredTerminalDiscovery,
    TerminalChannel,
};
