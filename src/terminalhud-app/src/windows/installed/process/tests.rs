use super::*;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn selected_command_keeps_quoting_environment_and_cwd() {
    let path = std::env::temp_dir().join(format!(
        "terminalhud-shell-fixture-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "windows::installed::process::tests::fixture_parent",
            "--ignored",
            "--nocapture",
        ])
        .current_dir(&path)
        .env("TER28_FIXTURE_ROOT", &path)
        .env("TER28_FIXTURE_VALUE", "a b & c")
        .output()
        .unwrap();
    assert!(output.status.success(), "fixture parent failed");
    assert_eq!(fs::read_to_string(path.join("result")).unwrap(), "a b & c");
    fs::remove_dir_all(path).unwrap();
}
#[test]
#[ignore = "spawned only by the selected-command fixture"]
fn fixture_parent() {
    let executable = std::env::current_exe().unwrap();
    let command = format!(
        "\"{}\" --exact windows::installed::process::tests::fixture_child --ignored --nocapture --skip \"pattern with spaces\"",
        executable.display()
    );
    assert_eq!(start(&command, "").unwrap().wait().unwrap(), 0);
    let failure = format!(
        "\"{}\" --exact windows::installed::process::tests::fixture_failure --ignored --nocapture",
        executable.display()
    );
    assert_eq!(start(&failure, "").unwrap().wait().unwrap(), 0xc000_013a);
}
#[test]
#[ignore = "spawned only by the selected-command fixture"]
fn fixture_child() {
    assert!(std::env::args().any(|arg| arg == "pattern with spaces"));
    let root = std::path::PathBuf::from(std::env::var_os("TER28_FIXTURE_ROOT").unwrap());
    assert_eq!(std::env::current_dir().unwrap(), root);
    fs::write(
        root.join("result"),
        std::env::var("TER28_FIXTURE_VALUE").unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "spawned only by the selected-command fixture"]
fn fixture_failure() {
    std::process::exit(0xc000_013au32 as i32);
}
