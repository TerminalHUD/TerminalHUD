use super::Result;
use std::{
    thread,
    time::{Duration, Instant},
};

pub(super) fn wait(mut condition: impl FnMut() -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !condition() {
        if Instant::now() >= deadline {
            return Err("native observation timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
