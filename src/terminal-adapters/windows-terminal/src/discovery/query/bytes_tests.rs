use super::*;
const OP: DiscoveryOperation = DiscoveryOperation::PackageIdentity;

#[test]
fn identity_storage_is_aligned_and_strings_are_decoded_only_inside_the_reported_prefix() {
    let buffer = bytes::query(|length, raw| {
        if raw.is_null() {
            *length = 16;
            return 122;
        }
        assert_eq!((raw as usize) % 8, 0);
        // SAFETY: the query supplied 16 initialized writable bytes after sizing.
        let bytes = unsafe { std::slice::from_raw_parts_mut(raw, 16) };
        bytes.fill(0);
        bytes[8..10].copy_from_slice(&97u16.to_ne_bytes());
        *length = 12;
        0
    })
    .unwrap();
    assert_eq!(buffer.bytes().len(), 12);
    let base = buffer.bytes().as_ptr();
    assert_eq!(buffer.string(base.wrapping_add(8).cast(), 8).unwrap(), "a");
    assert_eq!(buffer.string(base.wrapping_add(10).cast(), 8).unwrap(), "");
    for (address, minimum) in [
        (base as usize, 8),
        (base as usize + 9, 8),
        (base as usize + 12, 8),
        (0, 0),
    ] {
        assert!(matches!(
            buffer.string(address as *const u16, minimum),
            Err(DiscoveryFailure::Malformed {
                reason: MalformedReply::Pointer,
                ..
            })
        ));
    }
}

#[test]
fn identity_query_bounds_growth_and_rejects_inconsistent_success_sizes() {
    for (size, code, expected) in [
        (65_537, 122, DiscoveryFailure::Limit { operation: OP }),
        (0, 122, malformed(OP, MalformedReply::Size)),
        (1, 0, malformed(OP, MalformedReply::Size)),
        (
            0,
            1168,
            DiscoveryFailure::Native {
                operation: OP,
                code: 1168,
            },
        ),
    ] {
        assert!(
            matches!(bytes::query(|length,_| { *length=size;code }),Err(error) if error==expected)
        );
    }
    let mut calls = 0;
    assert!(matches!(
        bytes::query(|length, _| {
            calls += 1;
            *length = 16;
            122
        }),
        Err(DiscoveryFailure::ChangingResponse { .. })
    ));
    assert_eq!(calls, 4);
}
