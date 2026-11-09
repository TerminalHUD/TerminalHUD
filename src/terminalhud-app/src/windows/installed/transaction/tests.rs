use super::*;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: AtomicUsize = AtomicUsize::new(1);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "terminalhud-owned-patch-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(fixture: &Fixture, expected: &[u8], updated: &[u8]) -> io::Result<()> {
    apply(Plan {
        settings: &fixture.path("settings.json"),
        expected,
        updated,
        staged: &fixture.path("staged"),
        backup: &fixture.path("backup"),
        journal: &fixture.path("pending"),
        receipt: &fixture.path("receipt"),
        expected_receipt: None,
        new_receipt: b"owned",
        remove_receipt: false,
    })
}
#[test]
fn exact_patch_keeps_backup_and_receipt() {
    let fixture = Fixture::new();
    fs::write(fixture.path("settings.json"), b"original").unwrap();
    run(&fixture, b"original", b"updated").unwrap();
    assert_eq!(fs::read(fixture.path("settings.json")).unwrap(), b"updated");
    assert_eq!(fs::read(fixture.path("backup")).unwrap(), b"original");
    assert_eq!(fs::read(fixture.path("receipt")).unwrap(), b"owned");
    assert!(!fixture.path("pending").exists());
}
#[test]
fn stale_plan_and_existing_journal_never_change_settings() {
    let fixture = Fixture::new();
    fs::write(fixture.path("settings.json"), b"user edit").unwrap();
    assert!(run(&fixture, b"original", b"updated").is_err());
    assert_eq!(
        fs::read(fixture.path("settings.json")).unwrap(),
        b"user edit"
    );
    fs::write(fixture.path("pending"), b"unresolved operation").unwrap();
    assert!(run(&fixture, b"user edit", b"updated").is_err());
    assert_eq!(
        fs::read(fixture.path("settings.json")).unwrap(),
        b"user edit"
    );
    assert_eq!(
        fs::read(fixture.path("pending")).unwrap(),
        b"unresolved operation"
    );
}
#[test]
fn failed_receipt_preserves_replacement_backup_and_pending_journal() {
    let fixture = Fixture::new();
    fs::write(fixture.path("settings.json"), b"original").unwrap();
    FAIL_AFTER_WRITE.set(true);
    assert!(run(&fixture, b"original", b"updated").is_err());
    assert_eq!(fs::read(fixture.path("settings.json")).unwrap(), b"updated");
    assert_eq!(fs::read(fixture.path("backup")).unwrap(), b"original");
    assert_eq!(fs::read(fixture.path("pending")).unwrap(), b"owned");
    assert!(!fixture.path("receipt").exists());
}

#[test]
fn interruption_before_replace_keeps_original_and_pending_stage() {
    let fixture = Fixture::new();
    fs::write(fixture.path("settings.json"), b"original").unwrap();
    FAIL_BEFORE_REPLACE.set(true);
    assert!(run(&fixture, b"original", b"updated").is_err());
    assert_eq!(
        fs::read(fixture.path("settings.json")).unwrap(),
        b"original"
    );
    assert_eq!(fs::read(fixture.path("staged")).unwrap(), b"updated");
    assert_eq!(fs::read(fixture.path("pending")).unwrap(), b"owned");
    assert!(!fixture.path("backup").exists());
}
#[test]
fn racing_writer_is_saved_in_displaced_backup_and_not_rolled_over() {
    let fixture = Fixture::new();
    fs::write(fixture.path("settings.json"), b"original").unwrap();
    RACE_AFTER_CLOSE.set(true);
    assert!(run(&fixture, b"original", b"updated").is_err());
    assert_eq!(fs::read(fixture.path("settings.json")).unwrap(), b"updated");
    assert_eq!(
        fs::read(fixture.path("backup")).unwrap(),
        b"concurrent change"
    );
    assert_eq!(fs::read(fixture.path("pending")).unwrap(), b"owned");
    assert!(!fixture.path("receipt").exists());
}
#[test]
fn unchanged_receipt_can_be_updated_with_a_new_backup() {
    let fixture = Fixture::new();
    fs::write(fixture.path("settings.json"), b"original").unwrap();
    fs::write(fixture.path("receipt"), b"old-owned").unwrap();
    apply(Plan {
        settings: &fixture.path("settings.json"),
        expected: b"original",
        updated: b"updated",
        staged: &fixture.path("staged"),
        backup: &fixture.path("backup"),
        journal: &fixture.path("pending"),
        receipt: &fixture.path("receipt"),
        expected_receipt: Some(b"old-owned"),
        new_receipt: b"new-owned",
        remove_receipt: false,
    })
    .unwrap();
    assert_eq!(fs::read(fixture.path("settings.json")).unwrap(), b"updated");
    assert_eq!(fs::read(fixture.path("backup")).unwrap(), b"original");
    assert_eq!(fs::read(fixture.path("receipt")).unwrap(), b"new-owned");
}
#[test]
fn oversized_receipt_is_rejected_before_stage_or_settings_write() {
    let fixture = Fixture::new();
    fs::write(fixture.path("settings.json"), b"original").unwrap();
    let oversized = vec![b'x'; super::super::sources::MAX_BYTES + 1];
    assert!(
        apply(Plan {
            settings: &fixture.path("settings.json"),
            expected: b"original",
            updated: b"updated",
            staged: &fixture.path("staged"),
            backup: &fixture.path("backup"),
            journal: &fixture.path("pending"),
            receipt: &fixture.path("receipt"),
            expected_receipt: None,
            new_receipt: &oversized,
            remove_receipt: false,
        })
        .is_err()
    );
    assert_eq!(
        fs::read(fixture.path("settings.json")).unwrap(),
        b"original"
    );
    assert!(!fixture.path("staged").exists());
    assert!(!fixture.path("receipt").exists());
}
