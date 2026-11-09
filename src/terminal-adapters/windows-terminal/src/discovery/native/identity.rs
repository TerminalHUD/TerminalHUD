use super::super::{query::bytes, types::*};
use super::packages::wide;
use std::mem::{align_of, size_of};
use windows_sys::Win32::{
    Storage::Packaging::Appx::{PACKAGE_ID, PACKAGE_INFORMATION_BASIC, PackageIdFromFullName},
    System::SystemInformation::{
        PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_INTEL,
    },
};
const OP: DiscoveryOperation = DiscoveryOperation::PackageIdentity;

pub(super) fn identity(full_name: &str) -> Result<RegisteredPackageIdentity, DiscoveryFailure> {
    let name = wide(full_name);
    // The output allocation is u64-aligned before it is passed to the API.
    const {
        assert!(align_of::<u64>() >= align_of::<PACKAGE_ID>());
    }
    let buffer = bytes::query(|length, buffer| {
        // SAFETY: retained UTF-16 input; output is aligned owned storage of the declared byte capacity.
        unsafe { PackageIdFromFullName(name.as_ptr(), PACKAGE_INFORMATION_BASIC, length, buffer) }
    })?;
    decode(&buffer)
}

pub(super) fn decode(
    buffer: &bytes::IdentityBuffer,
) -> Result<RegisteredPackageIdentity, DiscoveryFailure> {
    if buffer.bytes().len() < size_of::<PACKAGE_ID>() {
        return Err(DiscoveryFailure::Malformed {
            operation: OP,
            reason: MalformedReply::Size,
        });
    }
    // SAFETY: validated initialized header prefix; copy it without retaining borrowed fields.
    // PACKAGE_ID is packed on x64/arm64, so read_unaligned handles its declared layout.
    let id = unsafe { std::ptr::read_unaligned(buffer.bytes().as_ptr().cast::<PACKAGE_ID>()) };
    let name = buffer.string(id.name, size_of::<PACKAGE_ID>())?;
    let publisher_id = buffer.string(id.publisherId, size_of::<PACKAGE_ID>())?;
    if name.is_empty() || publisher_id.is_empty() {
        return Err(DiscoveryFailure::Malformed {
            operation: OP,
            reason: MalformedReply::EmptyString,
        });
    }
    let resource_id = if id.resourceId.is_null() {
        String::new()
    } else {
        buffer.string(id.resourceId, size_of::<PACKAGE_ID>())?
    };
    // SAFETY: PACKAGE_VERSION's integer view is valid for every initialized version bit pattern.
    let version = unsafe { id.version.Anonymous.Version };
    let architecture = match id.processorArchitecture {
        n if n == u32::from(PROCESSOR_ARCHITECTURE_INTEL) => PackageArchitecture::X86,
        n if n == u32::from(PROCESSOR_ARCHITECTURE_AMD64) => PackageArchitecture::X64,
        n if n == u32::from(PROCESSOR_ARCHITECTURE_ARM64) => PackageArchitecture::Arm64,
        other => PackageArchitecture::Other(other),
    };
    Ok(RegisteredPackageIdentity {
        name,
        publisher_id,
        resource_id,
        package_version: [
            (version >> 48) as u16,
            (version >> 32) as u16,
            (version >> 16) as u16,
            version as u16,
        ],
        architecture,
    })
}
