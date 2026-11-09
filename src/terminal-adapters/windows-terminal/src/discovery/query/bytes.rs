use super::*;
const OP: DiscoveryOperation = DiscoveryOperation::PackageIdentity;

/// u64 allocation guarantees PACKAGE_ID's alignment on Windows x86/x64/arm64.
/// The API capacity is measured in bytes; the extra alignment padding is never parsed.
pub(in super::super) struct IdentityBuffer {
    storage: Vec<u64>,
    used: usize,
}
impl IdentityBuffer {
    pub fn bytes(&self) -> &[u8] {
        // SAFETY: initialized u64 storage; only its reported, validated used prefix is exposed.
        unsafe { std::slice::from_raw_parts(self.storage.as_ptr().cast(), self.used) }
    }
    pub fn string(&self, pointer: *const u16, minimum: usize) -> Result<String, DiscoveryFailure> {
        let start = offset(
            self.storage.as_ptr() as usize,
            self.used,
            pointer as usize,
            OP,
        )?;
        if start < minimum {
            return Err(malformed(OP, MalformedReply::Pointer));
        }
        let bytes = &self.bytes()[start..];
        let words: Vec<_> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| u16::from_ne_bytes([v[0], v[1]]))
            .collect();
        if words.first() == Some(&0) {
            Ok(String::new())
        } else {
            decode(&words, OP)
        }
    }
}

pub(in super::super) fn query(
    mut query: impl FnMut(&mut u32, *mut u8) -> u32,
) -> Result<IdentityBuffer, DiscoveryFailure> {
    let mut storage = Vec::<u64>::new();
    let mut capacity = 0;
    for _ in 0..ATTEMPTS {
        let mut length = capacity;
        let pointer = if storage.is_empty() {
            std::ptr::null_mut()
        } else {
            storage.as_mut_ptr().cast()
        };
        let code = query(&mut length, pointer);
        limit(OP, length as usize, MAX_UNITS)?;
        match code {
            SUCCESS => {
                if length > capacity {
                    return Err(malformed(OP, MalformedReply::Size));
                }
                return Ok(IdentityBuffer {
                    storage,
                    used: length as usize,
                });
            }
            INSUFFICIENT => {
                if length == 0 {
                    return Err(malformed(OP, MalformedReply::Size));
                }
                capacity = length;
                storage.resize((length as usize).div_ceil(8), u64::MAX);
                storage.fill(u64::MAX);
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
