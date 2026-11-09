pub(super) mod bytes;
pub(super) mod family;
pub(super) mod text;
use super::types::{DiscoveryFailure, DiscoveryOperation, MalformedReply};

const SUCCESS: u32 = 0;
const INSUFFICIENT: u32 = 122;
const ATTEMPTS: usize = 4;
const MAX_UNITS: usize = 65_536;
fn malformed(operation: DiscoveryOperation, reason: MalformedReply) -> DiscoveryFailure {
    DiscoveryFailure::Malformed { operation, reason }
}
fn limit(
    operation: DiscoveryOperation,
    amount: usize,
    maximum: usize,
) -> Result<(), DiscoveryFailure> {
    if amount > maximum {
        Err(DiscoveryFailure::Limit { operation })
    } else {
        Ok(())
    }
}
fn decode(words: &[u16], operation: DiscoveryOperation) -> Result<String, DiscoveryFailure> {
    let end = words
        .iter()
        .position(|v| *v == 0)
        .ok_or_else(|| malformed(operation, MalformedReply::Terminator))?;
    if end == 0 {
        return Err(malformed(operation, MalformedReply::EmptyString));
    }
    String::from_utf16(&words[..end]).map_err(|_| malformed(operation, MalformedReply::Utf16))
}
/// Convert a returned pointer to an offset; never dereference untrusted returned pointers.
fn offset(
    base: usize,
    used_bytes: usize,
    address: usize,
    operation: DiscoveryOperation,
) -> Result<usize, DiscoveryFailure> {
    address
        .checked_sub(base)
        .filter(|v| *v < used_bytes && v.is_multiple_of(2))
        .ok_or_else(|| malformed(operation, MalformedReply::Pointer))
}

#[cfg(test)]
mod bytes_tests;
#[cfg(test)]
mod family_tests;
#[cfg(test)]
mod text_tests;
