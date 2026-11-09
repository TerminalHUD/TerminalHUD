//! Validated portable product preferences, without native application effects.
mod error;
mod model;
mod profile;
mod shortcut;

pub use error::ValidationError;
pub use model::{Candidate, Configuration};
pub use profile::{ProfileId, ProfileSelection};
pub use shortcut::{Key, Modifier, Shortcut};

#[cfg(test)]
mod tests;
