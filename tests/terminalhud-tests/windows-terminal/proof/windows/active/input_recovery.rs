//! Balance only this probe's inserted button-down; never move the cursor back.
pub(crate) fn neutral(states: [i16; 5]) -> bool {
    states.into_iter().all(|state| state >= 0)
}

pub(crate) fn complete(
    inserted: u32,
    release_left: impl FnOnce() -> bool,
) -> Result<(), &'static str> {
    // SendInput inserts this MOVE, LEFTDOWN, LEFTUP sequence serially. Prefix
    // length two is the only outcome leaving our own LEFTDOWN unmatched.
    if inserted == 2 && !release_left() {
        return Err("owned click incomplete; compensating LEFTUP insertion failed");
    }
    if inserted != 3 {
        return Err("owned click SendInput was incomplete");
    }
    Ok(())
}
