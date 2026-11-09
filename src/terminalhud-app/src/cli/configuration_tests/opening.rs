use super::*;
use crate::config::{decode, encode};
use terminalhud_core::config::Configuration;

#[test]
fn explicit_open_creates_valid_defaults_and_passes_the_exact_path() {
    let fixture = Fixture::new();
    let response = run_at(ConfigCommand::Open, &fixture.path, |document| {
        assert_eq!(document, fixture.path);
        let bytes = fs::read(document).unwrap();
        let defaults = encode(&Configuration::default()).unwrap();
        assert_eq!(bytes, defaults.as_bytes());
        assert_eq!(
            decode(&String::from_utf8(bytes).unwrap()).unwrap(),
            Configuration::default()
        );
        Ok(())
    })
    .unwrap();
    assert!(response.contains(&fixture.path.display().to_string()));
}

#[test]
fn open_preserves_existing_invalid_bytes_and_reports_opener_failure() {
    let fixture = Fixture::new();
    let original = [0xff, b'{', b'X', b'}'];
    fixture.write(original);
    let error = run_at(ConfigCommand::Open, &fixture.path, |document| {
        assert_eq!(document, fixture.path);
        assert_eq!(fs::read(document).unwrap(), original);
        Err("no document association (fixture)".into())
    })
    .unwrap_err();
    assert!(error.contains("association"));
    assert_eq!(fs::read(&fixture.path).unwrap(), original);
    run_at(ConfigCommand::Open, &fixture.path, |_| Ok(())).unwrap();
    assert_eq!(fs::read(&fixture.path).unwrap(), original);
}

#[test]
fn concurrent_default_creation_does_not_truncate_an_existing_file() {
    let fixture = Fixture::new();
    let barrier = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                barrier.wait();
                crate::config::ensure_file(&fixture.path).unwrap();
            });
        }
    });
    let bytes = fs::read(&fixture.path).unwrap();
    assert_eq!(bytes, encode(&Configuration::default()).unwrap().as_bytes());
    fixture.write(b"user contents, invalid but preserved");
    crate::config::ensure_file(&fixture.path).unwrap();
    assert_eq!(
        fs::read(&fixture.path).unwrap(),
        b"user contents, invalid but preserved"
    );
}
