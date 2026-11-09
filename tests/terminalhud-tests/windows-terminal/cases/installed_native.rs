#![cfg(windows)]
use super::*;
#[test]
#[ignore = "read-only installed package/resource/provider observation; no Terminal launch or writes"]
fn installed_sources_can_be_read_without_loading_terminal() {
    let discovery = discover_registered_terminals();
    let candidates = discovery
        .channels
        .iter()
        .flat_map(|channel| channel.candidates.as_ref().into_iter().flatten());
    let mut checked = 0;
    for candidate in candidates {
        let Ok(executable) = &candidate.expected_executable else {
            continue;
        };
        let inbox = native::read_inbox_defaults(executable).unwrap();
        let catalog = Catalog::from_sources("{}", Some(&inbox), &[], &[]).unwrap();
        assert!(!catalog.profiles.is_empty());
        assert!(catalog.default_profile.is_some());
        checked += 1;
    }
    assert!(
        checked > 0,
        "installed Windows Terminal required for this explicit check"
    );
    let providers = native::discover_generated_profiles().unwrap();
    for provider in providers {
        assert!(
            provider.profile["commandline"]
                .as_str()
                .is_some_and(|c| !c.is_empty())
        );
        assert!(Guid::parse(provider.profile["guid"].as_str().unwrap()).is_ok());
    }
}
