use super::{candidate::candidate, types::*};
use std::path::PathBuf;
fn identity() -> RegisteredPackageIdentity {
    RegisteredPackageIdentity {
        name: TerminalChannel::Stable.package_name().into(),
        publisher_id: "8wekyb3d8bbwe".into(),
        resource_id: String::new(),
        package_version: [1, 24, 12741, 0],
        architecture: PackageArchitecture::X64,
    }
}
fn path() -> Result<PathBuf, DiscoveryFailure> {
    Ok(PathBuf::from("/supplied/metadata"))
}
#[test]
fn independent_query_errors_remain_partial_without_authenticating_other_paths() {
    let error = DiscoveryFailure::Native {
        operation: DiscoveryOperation::PackagePath,
        code: 5,
    };
    let report = candidate(
        TerminalChannel::Stable,
        "supplied".into(),
        Ok(identity()),
        Err(error.clone()),
        &path(),
    );
    assert!(report.identity.is_ok());
    assert!(report.expected_settings.is_ok());
    assert_eq!(report.expected_executable, Err(error));
    let folder_error = DiscoveryFailure::Native {
        operation: DiscoveryOperation::LocalAppData,
        code: 0x80004005,
    };
    let report = candidate(
        TerminalChannel::Stable,
        "supplied".into(),
        Ok(identity()),
        path(),
        &Err(folder_error.clone()),
    );
    assert!(report.expected_executable.is_ok());
    assert_eq!(report.expected_settings, Err(folder_error));
}
#[test]
fn resource_bundle_neutral_unknown_and_mismatched_identities_never_get_expected_app_paths() {
    for (resource, architecture) in [
        ("en-US", PackageArchitecture::X64),
        ("~", PackageArchitecture::Other(11)),
        ("", PackageArchitecture::Other(11)),
        ("", PackageArchitecture::Other(65535)),
    ] {
        let id = RegisteredPackageIdentity {
            resource_id: resource.into(),
            architecture,
            ..identity()
        };
        let report = candidate(
            TerminalChannel::Stable,
            "supplied".into(),
            Ok(id.clone()),
            path(),
            &path(),
        );
        assert_eq!(report.identity, Ok(id));
        assert_eq!(
            report.expected_executable,
            Err(DiscoveryFailure::UnsupportedApplicationPackage)
        );
        assert_eq!(
            report.expected_settings,
            Err(DiscoveryFailure::UnsupportedApplicationPackage)
        );
    }
    let report = candidate(
        TerminalChannel::Preview,
        "supplied".into(),
        Ok(identity()),
        path(),
        &path(),
    );
    assert!(matches!(
        report.identity,
        Err(DiscoveryFailure::Malformed {
            reason: MalformedReply::IdentityMismatch,
            ..
        })
    ));
}
#[test]
fn multiple_versions_remain_separate_candidates_with_package_not_runtime_versions() {
    let reports = [1, 2].map(|major| {
        candidate(
            TerminalChannel::Stable,
            format!("candidate{major}"),
            Ok(RegisteredPackageIdentity {
                package_version: [major, 0, 0, 0],
                ..identity()
            }),
            path(),
            &path(),
        )
    });
    assert_ne!(reports[0].full_name, reports[1].full_name);
    assert_eq!(
        reports[0].identity.as_ref().unwrap().package_version,
        [1, 0, 0, 0]
    );
    assert_eq!(
        reports[1].identity.as_ref().unwrap().package_version,
        [2, 0, 0, 0]
    );
}
