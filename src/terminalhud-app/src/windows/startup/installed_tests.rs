use super::*;
use serde_json::json;

struct Fixture {
    root: PathBuf,
    gui: PathBuf,
    receipt: serde_json::Value,
}
impl Fixture {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "terminalhud-startup-{}-{nonce}",
            std::process::id()
        ));
        let gui = root
            .join("versions")
            .join("build1")
            .join("terminalhud-shell.exe");
        fs::create_dir_all(gui.parent().unwrap()).unwrap();
        fs::write(&gui, b"fake GUI: never executed").unwrap();
        let command = format!("\"{}\"", gui.display());
        let receipt = json!({"schema_version":1,"product":"TerminalHUD","installed":true,"active_build":"build1","shortcuts":[],"versions":[{"build_id":"build1","files":[{"path":"versions/build1/terminalhud-shell.exe","sha256":"a".repeat(64)}]}],"startup_commands":[command]});
        Self { root, gui, receipt }
    }
    fn save(&self) {
        fs::write(
            self.root.join("receipt.json"),
            serde_json::to_vec(&self.receipt).unwrap(),
        )
        .unwrap();
    }
    fn load(&self) -> Result<Installed, StartupError> {
        Installed::load_at(self.gui.clone(), self.root.clone())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
#[test]
fn constructor_reads_receipted_gui_without_writing_or_executing() {
    let fixture = Fixture::new();
    fixture.save();
    let before = fs::read(fixture.root.join("receipt.json")).unwrap();
    let installed = fixture.load().unwrap();
    assert_eq!(installed.command, format!("\"{}\"", fixture.gui.display()));
    assert_eq!(installed.owned_commands, vec![installed.command]);
    assert_eq!(before, fs::read(fixture.root.join("receipt.json")).unwrap());
    assert_eq!(fs::read(&fixture.gui).unwrap(), b"fake GUI: never executed");
}
#[test]
fn inactive_or_superseded_install_cannot_reenable_startup() {
    let mut fixture = Fixture::new();
    fixture.receipt["installed"] = json!(false);
    fixture.save();
    assert!(fixture.load().is_err());
    fixture.receipt["installed"] = json!(true);
    fixture.receipt["active_build"] = json!("newbuild");
    fixture.save();
    assert!(fixture.load().is_err());
}
#[test]
fn missing_receipt_foreign_command_malformed_hash_and_outside_path_are_refused() {
    let mut fixture = Fixture::new();
    assert!(fixture.load().is_err());
    fixture.receipt["startup_commands"] =
        json!([format!("\"{}\" --prototype", fixture.gui.display())]);
    fixture.save();
    assert!(fixture.load().is_err());
    fixture.receipt["startup_commands"] = json!([format!("\"{}\"", fixture.gui.display())]);
    fixture.receipt["versions"][0]["files"][0]["sha256"] = json!("z".repeat(64));
    fixture.save();
    assert!(fixture.load().is_err());
    assert!(Installed::load_at(fixture.root.join("outside.exe"), fixture.root.clone()).is_err());
}
