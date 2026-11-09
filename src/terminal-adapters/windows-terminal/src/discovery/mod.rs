//! Read-only observations of registered package candidates; no launch or catalog authority.
#[cfg(any(windows, test))]
mod candidate;
#[cfg(windows)]
mod native;
#[cfg(any(windows, test))]
mod query;
mod types;
#[cfg(windows)]
pub use native::discover_registered_terminals;
pub use types::{
    DiscoveryFailure, DiscoveryOperation, MalformedReply, PackageArchitecture, RegisteredChannel,
    RegisteredPackageCandidate, RegisteredPackageIdentity, RegisteredTerminalDiscovery,
    TerminalChannel,
};
#[cfg(test)]
mod candidate_tests;
