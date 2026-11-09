use super::*;
const MAX_PATH_UNITS: usize = 32_768;

pub(in super::super) fn query(
    operation: DiscoveryOperation,
    mut query: impl FnMut(&mut u32, &mut [u16]) -> u32,
) -> Result<String, DiscoveryFailure> {
    let mut buffer = Vec::new();
    for _ in 0..ATTEMPTS {
        let mut length = buffer.len() as u32;
        let code = query(&mut length, &mut buffer);
        limit(operation, length as usize, MAX_PATH_UNITS)?;
        match code {
            SUCCESS => {
                let used = buffer
                    .get(..length as usize)
                    .ok_or_else(|| malformed(operation, MalformedReply::Size))?;
                return decode(used, operation);
            }
            INSUFFICIENT => {
                if length == 0 {
                    return Err(malformed(operation, MalformedReply::Size));
                }
                buffer.resize(length as usize, 0xffff);
                buffer.fill(0xffff);
            }
            _ => return Err(DiscoveryFailure::Native { operation, code }),
        }
    }
    Err(DiscoveryFailure::ChangingResponse { operation })
}
