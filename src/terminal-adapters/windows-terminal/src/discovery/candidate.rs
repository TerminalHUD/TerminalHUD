use super::types::*;
use std::path::PathBuf;

pub(super) fn candidate(
    channel: TerminalChannel,
    full_name: String,
    identity: Result<RegisteredPackageIdentity, DiscoveryFailure>,
    registered_install_path: Result<PathBuf, DiscoveryFailure>,
    local_app_data: &Result<PathBuf, DiscoveryFailure>,
) -> RegisteredPackageCandidate {
    let identity = identity.and_then(|id| {
        if id.name != channel.package_name() || id.publisher_id != "8wekyb3d8bbwe" {
            Err(DiscoveryFailure::Malformed {
                operation: DiscoveryOperation::PackageIdentity,
                reason: MalformedReply::IdentityMismatch,
            })
        } else {
            Ok(id)
        }
    });
    let application = identity.as_ref().map_err(Clone::clone).and_then(|id| {
        if id.resource_id.is_empty()
            && matches!(
                id.architecture,
                PackageArchitecture::X86 | PackageArchitecture::X64 | PackageArchitecture::Arm64
            )
        {
            Ok(())
        } else {
            Err(DiscoveryFailure::UnsupportedApplicationPackage)
        }
    });
    let expected_executable = application
        .clone()
        .and_then(|_| registered_install_path.clone())
        .map(|path| path.join("WindowsTerminal.exe"));
    let expected_settings = application
        .and_then(|_| local_app_data.clone())
        .map(|path| {
            path.join("Packages")
                .join(channel.family())
                .join("LocalState")
                .join("settings.json")
        });
    RegisteredPackageCandidate {
        channel,
        family: channel.family().into(),
        full_name,
        identity,
        registered_install_path,
        expected_executable,
        expected_settings,
    }
}
