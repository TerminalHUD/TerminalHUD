//! Portable HudSwitch state machine and synchronous backend contract.
//!
//! A controller exclusively owns its target and backend. Hosts serialize event
//! delivery through a queue or a mutex; [`Controller::dispatch`] requires an
//! exclusive borrow for the complete transition. Backend callbacks enqueue new
//! events rather than recursively dispatching them. See `docs/hudswitch-core.md`.

#![forbid(unsafe_code)]

mod backend;
mod capabilities;
mod controller;
mod error;
mod observation;
mod state;
mod status;

pub use backend::{Acquired, Backend};
pub use capabilities::Capabilities;
pub use controller::{Controller, TargetId};
pub use error::{AttemptFailure, Cause, Outcome, Recovery, Stage, TransitionError};
pub use observation::Observation;
pub use state::{Presentation, State, VisibleState};
pub use status::{Availability, Event, Status};
