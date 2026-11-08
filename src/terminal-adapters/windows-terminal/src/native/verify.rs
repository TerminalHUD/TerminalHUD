use super::{NativeError, ProcessIdentity, ancestry, windows};
use crate::{LaunchIdentity, SessionProof, correlate};
use std::collections::HashSet;
use std::os::windows::io::OwnedHandle;
use std::path::Path;
pub struct VerifiedWindow {
    window: isize,
    owner: ProcessIdentity,
}
impl VerifiedWindow {
    pub fn window_handle(&self) -> isize {
        self.window
    }
    pub fn process_id(&self) -> u32 {
        self.owner.pid
    }
    /// Composition may pass these to its backend's explicit verified-launch bridge.
    pub fn into_parts(self) -> (isize, OwnedHandle) {
        (self.window, self.owner.process)
    }
}

pub fn verify(
    identity: &LaunchIdentity,
    proof: &SessionProof,
    before: &HashSet<isize>,
    terminal_executable: &Path,
) -> Result<VerifiedWindow, NativeError> {
    let mut candidates = windows()?;
    for candidate in &mut candidates {
        candidate.existed_before_launch = before.contains(&candidate.handle);
    }
    let mut chain = ancestry(proof.shell_pid)?;
    let ids: Vec<_> = chain.iter().map(|entry| entry.pid).collect();
    let candidate =
        correlate(identity, proof, &ids, &candidates).map_err(NativeError::Correlation)?;
    let index = chain
        .iter()
        .position(|owner| owner.pid == candidate.process_id)
        .ok_or(NativeError::Invalid(
            "window owner is outside session ancestry",
        ))?;
    let owner = chain.swap_remove(index);
    if !owner
        .executable
        .to_string_lossy()
        .eq_ignore_ascii_case(&terminal_executable.to_string_lossy())
    {
        return Err(NativeError::Invalid(
            "window executable does not match installed Terminal",
        ));
    }
    // Re-read the current candidate before handing its verified process/window to composition.
    if !windows()?.iter().any(|current| {
        current.handle == candidate.handle
            && current.process_id == owner.pid
            && current.title == identity.title()
    }) {
        return Err(NativeError::Invalid(
            "Terminal window changed during verification",
        ));
    }
    Ok(VerifiedWindow {
        window: candidate.handle,
        owner,
    })
}
