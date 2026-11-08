use super::Result;
use std::{fs::OpenOptions, io::Write, path::Path};
pub(super) fn write_new(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}
