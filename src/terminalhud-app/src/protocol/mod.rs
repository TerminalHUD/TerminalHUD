//! Versioned fixed-size prototype frames, independent of transport mechanics.
mod reply;
mod request;
pub use reply::{Reply, ReplyCode};
pub use request::{decode_request, encode_request};
pub const REQUEST_LEN: usize = 8;
pub const REPLY_LEN: usize = 32;
#[cfg(test)]
mod tests;
