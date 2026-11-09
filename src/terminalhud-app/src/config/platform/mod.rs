#[cfg(windows)]
mod apartment;
#[cfg(windows)]
mod location;
#[cfg(windows)]
mod opening;
#[cfg(windows)]
pub(crate) use location::settings_path;
#[cfg(windows)]
pub(crate) use opening::open_document;

#[cfg(not(windows))]
pub(crate) fn settings_path() -> Result<std::path::PathBuf, String> {
    Err("configuration file commands currently require native Windows".into())
}
#[cfg(not(windows))]
pub(crate) fn open_document(_: &std::path::Path) -> Result<(), String> {
    Err("opening a configuration document currently requires native Windows".into())
}
