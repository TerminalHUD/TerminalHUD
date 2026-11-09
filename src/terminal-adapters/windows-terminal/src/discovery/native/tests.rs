use super::*;

#[test]
fn read_only_current_user_enumeration_does_not_require_terminal_installed() {
    let report = discover_registered_terminals();
    assert_eq!(report.channels[0].channel, TerminalChannel::Stable);
    assert_eq!(report.channels[1].channel, TerminalChannel::Preview);
    assert!(report.local_app_data.as_ref().unwrap().is_absolute());
    for channel in report.channels {
        // Empty is valid; do not require any installed Terminal or launch anything.
        let candidates = channel.candidates.unwrap();
        for candidate in candidates {
            assert_eq!(candidate.family, channel.channel.family());
            assert!(!candidate.full_name.is_empty());
        }
    }
}

#[test]
fn basic_identity_api_decodes_a_synthetic_name_without_loading_or_installing_a_package() {
    let id =
        identity::identity("Microsoft.WindowsTerminal_1.24.12741.0_x64__8wekyb3d8bbwe").unwrap();
    assert_eq!(id.package_version, [1, 24, 12741, 0]);
    assert_eq!(id.architecture, PackageArchitecture::X64);
    assert_eq!(id.name, TerminalChannel::Stable.package_name());
    assert_eq!(id.publisher_id, "8wekyb3d8bbwe");
    assert!(id.resource_id.is_empty());
}

#[test]
fn malformed_native_identity_headers_and_embedded_pointers_are_rejected() {
    use crate::discovery::query::bytes;
    use std::mem::size_of;
    use windows_sys::Win32::Storage::Packaging::Appx::PACKAGE_ID;
    for length in [1, size_of::<PACKAGE_ID>()] {
        let buffer = bytes::query(|size, raw| {
            if raw.is_null() {
                *size = length as u32;
                return 122;
            }
            // SAFETY: query supplied at least the requested initialized length. All-zero
            // header has NULL embedded pointers, which must be rejected without dereference.
            unsafe { std::ptr::write_bytes(raw, 0, length) };
            *size = length as u32;
            0
        })
        .unwrap();
        assert!(matches!(
            identity::decode(&buffer),
            Err(DiscoveryFailure::Malformed { .. })
        ));
    }
}
