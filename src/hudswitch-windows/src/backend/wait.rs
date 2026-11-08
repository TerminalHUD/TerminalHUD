use super::*;

pub(super) fn wait_until(
    target: &Target,
    operation: &'static str,
    mut condition: impl FnMut() -> Result<bool, Error>,
) -> Result<(), Error> {
    let deadline = Instant::now() + WAIT;
    loop {
        target.ensure()?;
        if condition()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(Error::Timeout(operation));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
