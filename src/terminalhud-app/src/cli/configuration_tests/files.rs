use super::*;

#[test]
fn validation_accepts_lf_and_crlf_without_changing_bytes() {
    let fixture = Fixture::new();
    for json in ["{\n  \"version\": 1\n}", "{\r\n  \"version\": 1\r\n}"] {
        fixture.write(json);
        let response = run_at(ConfigCommand::Validate, &fixture.path, |_| {
            panic!("no opener")
        })
        .unwrap();
        assert!(response.contains("Valid configuration"));
        assert_eq!(fs::read(&fixture.path).unwrap(), json.as_bytes());
    }
}

#[test]
fn validation_rejects_invalid_files_without_repairing_them() {
    let fixture = Fixture::new();
    for bytes in [
        b"broken".as_slice(),
        b"{}",
        b"{\"version\":2}",
        b"{\"version\":1,\"version\":1}",
        b"{\"version\":1,\"terminal\":{\"activeBackgroundOpacity\":101}}",
        &[0xff],
    ] {
        fixture.write(bytes);
        assert!(
            run_at(ConfigCommand::Validate, &fixture.path, |_| panic!(
                "no opener"
            ))
            .is_err()
        );
        assert_eq!(fs::read(&fixture.path).unwrap(), bytes);
    }
    let oversized = vec![b' '; 1024 * 1024 + 1];
    fixture.write(&oversized);
    let error = run_at(ConfigCommand::Validate, &fixture.path, |_| {
        panic!("no opener")
    })
    .unwrap_err();
    assert!(error.contains("1 MiB"));
    assert_eq!(fs::read(&fixture.path).unwrap(), oversized);
}

#[test]
fn directories_are_rejected_for_validation_and_open() {
    let fixture = Fixture::new();
    fs::create_dir_all(&fixture.path).unwrap();
    for command in [ConfigCommand::Validate, ConfigCommand::Open] {
        let error = run_at(command, &fixture.path, |_| panic!("no opener")).unwrap_err();
        assert!(error.contains("regular settings file"));
    }
}

#[cfg(unix)]
#[test]
fn symlinks_are_rejected_without_touching_the_target() {
    let fixture = Fixture::new();
    let target = fixture.root.join("original.json");
    fs::write(&target, b"original").unwrap();
    fs::create_dir_all(fixture.path.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, &fixture.path).unwrap();
    for command in [ConfigCommand::Validate, ConfigCommand::Open] {
        assert!(run_at(command, &fixture.path, |_| panic!("no opener")).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"original");
    }
}

#[cfg(windows)]
#[test]
fn unreadable_file_reports_an_error_without_modifying_contents() {
    use std::os::windows::fs::OpenOptionsExt;
    let fixture = Fixture::new();
    fixture.write(b"{\"version\":1}");
    let exclusive = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&fixture.path)
        .unwrap();
    let error = run_at(ConfigCommand::Validate, &fixture.path, |_| {
        panic!("no opener")
    })
    .unwrap_err();
    assert!(error.contains("cannot read configuration"));
    drop(exclusive);
    assert_eq!(fs::read(&fixture.path).unwrap(), b"{\"version\":1}");
}
