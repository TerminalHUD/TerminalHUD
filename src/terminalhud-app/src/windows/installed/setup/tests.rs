use super::*;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use terminalhud_core::config::Candidate;
static NEXT: AtomicUsize = AtomicUsize::new(1);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "terminalhud-profile-fixture-{}-{}-{}",
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
    fn sources(&self) -> Sources {
        Sources {
            executable: self.0.join("terminal.exe"),
            settings: self.0.join("settings.json"),
            local_app_data: self.0.clone(),
            fragments: vec![],
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
const SETTINGS: &str = "{\n // preserve me\n \"defaultProfile\":\"{11111111-1111-1111-1111-111111111111}\",\n \"profiles\":{\"list\":[{\"guid\":\"{11111111-1111-1111-1111-111111111111}\",\"name\":\"fixture\",\"commandline\":\"cmd.exe /K\",\"font\":{\"face\":\"Fixture Face\"}}]}\n}\n";
#[test]
fn setup_and_reload_preserve_ordinary_bytes_and_refuse_changed_owned_entry() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("settings.json"), SETTINGS).unwrap();
    let first = prepare_from(&Configuration::default(), fixture.sources(), None, &[]).unwrap();
    assert!(!first.is_owned());
    let guid = first.profile.hud_guid.clone();
    first.commit().unwrap();
    let after = fs::read_to_string(fixture.0.join("settings.json")).unwrap();
    assert_eq!(
        remove_owned_profile(&after, &first.patch.ownership).unwrap(),
        SETTINGS
    );
    let unchanged = prepare_from(&Configuration::default(), fixture.sources(), None, &[]).unwrap();
    assert!(unchanged.is_owned());
    assert_eq!(unchanged.profile.hud_guid, guid);
    let config = Configuration::validate(Candidate {
        active_background_opacity: 60,
        ..Candidate::default()
    })
    .unwrap();
    let changed = prepare_from(&config, fixture.sources(), None, &[]).unwrap();
    changed.commit().unwrap();
    let after = fs::read_to_string(fixture.0.join("settings.json")).unwrap();
    assert_eq!(
        remove_owned_profile(&after, &changed.patch.ownership).unwrap(),
        SETTINGS
    );
    assert_eq!(changed.profile.hud_guid, guid);
    fs::write(
        fixture.0.join("settings.json"),
        after.replace("Fixture Face", "User Edit"),
    )
    .unwrap();
    assert!(prepare_from(&config, fixture.sources(), None, &[]).is_err());
}
