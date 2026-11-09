use super::*;
use std::collections::HashSet;
use std::ptr::null_mut;
const OP: DiscoveryOperation = DiscoveryOperation::PackageFamily;
const MAX_PACKAGES: usize = 128;

pub(in super::super) fn names(
    mut query: impl FnMut(&mut u32, &mut [*mut u16], &mut u32, &mut [u16]) -> u32,
) -> Result<Vec<String>, DiscoveryFailure> {
    let mut pointers = Vec::new();
    let mut buffer = Vec::new();
    for _ in 0..ATTEMPTS {
        let mut count = pointers.len() as u32;
        let mut length = buffer.len() as u32;
        let code = query(&mut count, &mut pointers, &mut length, &mut buffer);
        limit(OP, count as usize, MAX_PACKAGES)?;
        limit(OP, length as usize, MAX_UNITS)?;
        match code {
            SUCCESS => return decode_names(count as usize, length as usize, &pointers, &buffer),
            INSUFFICIENT => {
                if count == 0 || length == 0 {
                    return Err(malformed(OP, MalformedReply::Size));
                }
                pointers.resize(count as usize, null_mut());
                pointers.fill(null_mut());
                buffer.resize(length as usize, 0xffff);
                buffer.fill(0xffff);
            }
            _ => {
                return Err(DiscoveryFailure::Native {
                    operation: OP,
                    code,
                });
            }
        }
    }
    Err(DiscoveryFailure::ChangingResponse { operation: OP })
}

fn decode_names(
    count: usize,
    length: usize,
    pointers: &[*mut u16],
    buffer: &[u16],
) -> Result<Vec<String>, DiscoveryFailure> {
    if count > pointers.len() {
        return Err(malformed(OP, MalformedReply::Count));
    }
    if length > buffer.len() {
        return Err(malformed(OP, MalformedReply::Size));
    }
    let buffer = &buffer[..length];
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for &pointer in &pointers[..count] {
        let start = offset(
            buffer.as_ptr() as usize,
            buffer.len() * 2,
            pointer as usize,
            OP,
        )? / 2;
        let name = decode(&buffer[start..], OP)?;
        if !seen.insert(name.clone()) {
            return Err(malformed(OP, MalformedReply::DuplicateName));
        }
        result.push(name);
    }
    Ok(result)
}
