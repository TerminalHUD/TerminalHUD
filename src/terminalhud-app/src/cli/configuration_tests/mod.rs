use super::configuration::{ConfigCommand, run_at};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

mod files;
mod opening;
mod routing;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    path: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "terminalhud-config-{}-{stamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        Self {
            path: root.join("配置 space & data").join("settings.json"),
            root,
        }
    }
    fn write(&self, bytes: impl AsRef<[u8]>) {
        fs::create_dir_all(self.path.parent().unwrap()).unwrap();
        fs::write(&self.path, bytes).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
