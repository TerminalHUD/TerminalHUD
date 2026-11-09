use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
    pub schema_version: u32,
    pub product: String,
    pub installed: bool,
    pub active_build: String,
    pub shortcuts: Vec<Shortcut>,
    pub versions: Vec<Version>,
    pub startup_commands: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Version {
    pub build_id: String,
    pub files: Vec<File>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct File {
    pub path: String,
    pub sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Shortcut {
    pub path: String,
    pub sha256: String,
    pub target: String,
}
impl Receipt {
    pub fn owns_gui(&self, build: &str) -> bool {
        self.schema_version == 1
            && self.product == "TerminalHUD"
            && self.installed
            && self.active_build == build
            && self.shortcuts.iter().all(|s| {
                !s.path.is_empty()
                    && valid_hash(&s.sha256)
                    && self.startup_commands.contains(&s.target)
            })
            && self.versions.iter().any(|v| {
                v.build_id == build
                    && v.files.iter().any(|f| {
                        f.path == format!("versions/{build}/terminalhud-shell.exe")
                            && valid_hash(&f.sha256)
                    })
            })
    }
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
