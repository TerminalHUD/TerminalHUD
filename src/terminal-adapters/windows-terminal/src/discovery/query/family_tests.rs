use super::*;

#[test]
fn absent_registration_is_empty_and_api_errors_are_preserved() {
    assert!(
        family::names(|count, pointers, length, buffer| {
            assert_eq!((*count, *length), (0, 0));
            assert!(pointers.is_empty() && buffer.is_empty());
            0
        })
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        family::names(|_, _, _, _| 5),
        Err(DiscoveryFailure::Native {
            operation: DiscoveryOperation::PackageFamily,
            code: 5
        })
    );
}

#[test]
fn a_changing_registration_query_retries_and_preserves_all_names_in_order() {
    let mut calls = 0;
    let result = family::names(|count, pointers, length, buffer| {
        calls += 1;
        if calls == 1 {
            *count = 1;
            *length = 4;
            return 122;
        }
        if calls == 2 {
            *count = 2;
            *length = 8;
            return 122;
        }
        buffer.copy_from_slice(&[111, 110, 101, 0, 116, 119, 111, 0]);
        pointers[0] = buffer.as_mut_ptr();
        pointers[1] = buffer.as_mut_ptr().wrapping_add(4);
        *count = 2;
        *length = 8;
        0
    })
    .unwrap();
    assert_eq!(calls, 3);
    assert_eq!(result, ["one", "two"]);
}

#[test]
fn growth_and_retry_caps_refuse_incomplete_results() {
    assert!(matches!(
        family::names(|count, lengths, length, _| {
            assert!(lengths.is_empty());
            *count = 129;
            *length = 1024;
            122
        }),
        Err(DiscoveryFailure::Limit { .. })
    ));
    let mut calls = 0;
    assert!(matches!(
        family::names(|count, _, length, _| {
            calls += 1;
            *count = 1;
            *length = 8;
            122
        }),
        Err(DiscoveryFailure::ChangingResponse { .. })
    ));
    assert_eq!(calls, 4);
    assert!(matches!(
        family::names(|count, _, length, _| {
            *count = 1;
            *length = 65_537;
            122
        }),
        Err(DiscoveryFailure::Limit { .. })
    ));
}

#[test]
fn malformed_pointer_counts_lengths_utf16_and_duplicate_names_are_not_read_as_success() {
    for case in 0..8 {
        let result = family::names(|count, pointers, length, buffer| {
            if pointers.is_empty() {
                *count = 2;
                *length = 4;
                return 122;
            }
            buffer.copy_from_slice(&[97, 0, 98, 0]);
            pointers[0] = buffer.as_mut_ptr();
            pointers[1] = buffer.as_mut_ptr().wrapping_add(2);
            *count = 2;
            *length = 4;
            match case {
                0 => *count = 3,
                1 => *length = 5,
                2 => pointers[0] = std::ptr::null_mut(),
                3 => pointers[0] = buffer.as_mut_ptr().wrapping_add(4),
                4 => pointers[0] = buffer.as_mut_ptr().cast::<u8>().wrapping_add(1).cast(),
                5 => buffer.fill(97),
                6 => buffer[0] = 0xd800,
                7 => pointers[1] = pointers[0],
                _ => unreachable!(),
            }
            0
        });
        assert!(
            matches!(result, Err(DiscoveryFailure::Malformed { .. })),
            "case{case}: {result:?}"
        );
    }
}
