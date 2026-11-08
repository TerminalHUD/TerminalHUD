use super::LaunchIdentity;
use crate::{Error, Guid};
/// Combine correlation evidence; title/class or launcher PID alone is insufficient.
#[derive(Clone, Debug)]
pub struct SessionProof {
    pub nonce: Guid,
    pub profile: Guid,
    pub session: Guid,
    pub shell_pid: u32,
}
#[derive(Clone, Debug)]
pub struct WindowCandidate {
    pub handle: isize,
    pub process_id: u32,
    pub class: String,
    pub title: String,
    pub existed_before_launch: bool,
}

/// `ancestors` must come from retained/live process identity checks by the native
/// caller, not untrusted shell text. Exact executable verification is also required.
pub fn correlate<'a>(
    identity: &LaunchIdentity,
    proof: &SessionProof,
    ancestors: &[u32],
    candidates: &'a [WindowCandidate],
) -> Result<&'a WindowCandidate, Error> {
    if proof.nonce != identity.nonce || proof.profile != identity.profile || proof.shell_pid == 0 {
        return Err(Error::Invalid("session handshake does not match launch"));
    }
    let mut matches = candidates.iter().filter(|candidate| {
        !candidate.existed_before_launch
            && candidate.class == "CASCADIA_HOSTING_WINDOW_CLASS"
            && candidate.title == identity.title()
            && ancestors.contains(&candidate.process_id)
    });
    let candidate = matches.next().ok_or(Error::NoMatchingWindow)?;
    if matches.next().is_some() {
        return Err(Error::Invalid("ambiguous Terminal launch identity"));
    }
    Ok(candidate)
}
