use hudswitch_core::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

mod fake;
use fake::{OTHER_STYLE, assert_view, setup};

mod capabilities;
mod creation;
mod focus;
mod launch;
mod lifecycle;
mod recovery;
mod recovery_focus;
mod serialization;
mod shutdown;
mod transitions;
