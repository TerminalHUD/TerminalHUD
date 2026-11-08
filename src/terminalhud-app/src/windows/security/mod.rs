//! User/session identity and explicit pipe access control.
mod descriptor;
mod identity;
mod peer;
mod token;
pub(crate) use descriptor::Descriptor;
pub use identity::Identity;
pub(crate) use peer::verify_peer;
