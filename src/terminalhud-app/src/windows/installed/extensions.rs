//! Discover only declared current-user Terminal settings app extensions.
use std::{
    io,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use windows::{
    ApplicationModel::AppExtensions::AppExtensionCatalog,
    Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
    core::HSTRING,
};

static RUNNING: AtomicBool = AtomicBool::new(false);
struct Worker;
impl Drop for Worker {
    fn drop(&mut self) {
        RUNNING.store(false, Ordering::Release);
    }
}
struct Apartment;
impl Apartment {
    fn new() -> io::Result<Self> {
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(io::Error::other)?;
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            RoUninitialize();
        }
    }
}

pub(super) fn discover() -> io::Result<Vec<(String, PathBuf)>> {
    if RUNNING.swap(true, Ordering::AcqRel) {
        return Err(io::Error::other(
            "Terminal extension discovery is still running",
        ));
    }
    let (sender, receiver) = mpsc::sync_channel(1);
    if let Err(error) = thread::Builder::new()
        .name("terminalhud-extensions".into())
        .spawn(move || {
            let worker = Worker;
            let result = read_catalog();
            drop(worker);
            let _ = sender.send(result);
        })
    {
        RUNNING.store(false, Ordering::Release);
        return Err(error);
    }
    receiver
        .recv_timeout(Duration::from_secs(10))
        .map_err(|error| match error {
            mpsc::RecvTimeoutError::Timeout => {
                io::Error::other("Terminal extension discovery timed out")
            }
            mpsc::RecvTimeoutError::Disconnected => {
                io::Error::other("Terminal extension discovery worker failed")
            }
        })?
}
fn read_catalog() -> io::Result<Vec<(String, PathBuf)>> {
    let _apartment = Apartment::new()?;
    let catalog =
        AppExtensionCatalog::Open(&HSTRING::from("com.microsoft.windows.terminal.settings"))
            .map_err(io::Error::other)?;
    let extensions = catalog
        .FindAllAsync()
        .map_err(io::Error::other)?
        .join()
        .map_err(io::Error::other)?;
    let count = extensions.Size().map_err(io::Error::other)?;
    if count > 256 {
        return Err(io::Error::other("too many Terminal app extensions"));
    }
    let mut result = Vec::new();
    for index in 0..count {
        let extension = extensions.GetAt(index).map_err(io::Error::other)?;
        let namespace = extension
            .Package()
            .and_then(|package| package.Id())
            .and_then(|id| id.FamilyName())
            .map_err(io::Error::other)?
            .to_string();
        let folder = PathBuf::from(
            extension
                .GetPublicPath()
                .map_err(io::Error::other)?
                .to_string(),
        );
        if namespace.is_empty() || !folder.is_absolute() {
            return Err(io::Error::other("invalid Terminal app extension identity"));
        }
        result.push((namespace, folder.join("Fragments")));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn installed_catalog_observation_is_read_only_and_bounded() {
        let entries = super::discover().unwrap();
        assert!(entries.len() <= 256);
        assert!(
            entries.iter().all(|(namespace, path)| !namespace.is_empty()
                && path.is_absolute()
                && path.is_dir())
        );
    }
}
