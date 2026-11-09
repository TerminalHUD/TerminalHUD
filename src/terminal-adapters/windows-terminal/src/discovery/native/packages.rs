use super::super::{query, types::*};
use std::{path::PathBuf, ptr::null_mut};
use windows_sys::Win32::Storage::Packaging::Appx::{
    GetPackagePathByFullName, GetPackagesByPackageFamily,
};

pub(super) fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub(super) fn names(channel: TerminalChannel) -> Result<Vec<String>, DiscoveryFailure> {
    let family = wide(channel.family());
    query::family::names(|count, pointers, length, buffer| {
        let pointers = if pointers.is_empty() {
            null_mut()
        } else {
            pointers.as_mut_ptr()
        };
        let buffer = if buffer.is_empty() {
            null_mut()
        } else {
            buffer.as_mut_ptr()
        };
        // SAFETY: NUL-terminated family and writable buffers with capacities supplied by the query loop.
        unsafe { GetPackagesByPackageFamily(family.as_ptr(), count, pointers, length, buffer) }
    })
}
pub(super) fn path(full_name: &str) -> Result<PathBuf, DiscoveryFailure> {
    let name = wide(full_name);
    let path = query::text::query(DiscoveryOperation::PackagePath, |length, buffer| {
        let buffer = if buffer.is_empty() {
            null_mut()
        } else {
            buffer.as_mut_ptr()
        };
        // SAFETY: retained name and writable UTF-16 buffer of the declared capacity.
        unsafe { GetPackagePathByFullName(name.as_ptr(), length, buffer) }
    })?;
    absolute_path(path, DiscoveryOperation::PackagePath)
}
pub(super) fn absolute_path(
    path: String,
    operation: DiscoveryOperation,
) -> Result<PathBuf, DiscoveryFailure> {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(DiscoveryFailure::Malformed {
            operation,
            reason: MalformedReply::RelativePath,
        })
    }
}
