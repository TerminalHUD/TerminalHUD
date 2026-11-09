use super::*;
const OP: DiscoveryOperation = DiscoveryOperation::PackagePath;

#[test]
fn path_query_retries_growth_and_uses_only_the_successful_returned_length() {
    let mut calls = 0;
    let result = text::query(OP, |length, buffer| {
        calls += 1;
        if calls < 3 {
            *length = calls * 4;
            return 122;
        }
        buffer[..4].copy_from_slice(&[97, 98, 99, 0]);
        *length = 4;
        0
    })
    .unwrap();
    assert_eq!(result, "abc");
    assert_eq!(calls, 3);
}

#[test]
fn path_limits_errors_and_malformed_strings_are_explicit() {
    for (returned, code, expected) in [
        (32_769, 122, DiscoveryFailure::Limit { operation: OP }),
        (0, 122, malformed(OP, MalformedReply::Size)),
        (
            0,
            5,
            DiscoveryFailure::Native {
                operation: OP,
                code: 5,
            },
        ),
    ] {
        assert_eq!(
            text::query(OP, |length, _| {
                *length = returned;
                code
            }),
            Err(expected)
        );
    }
    for words in [vec![0], vec![97, 98], vec![0xd800, 0]] {
        let result = text::query(OP, |length, buffer| {
            if buffer.is_empty() {
                *length = words.len() as u32;
                return 122;
            }
            buffer.copy_from_slice(&words);
            0
        });
        assert!(matches!(result, Err(DiscoveryFailure::Malformed { .. })));
    }
    assert!(matches!(
        text::query(OP, |length, _| {
            *length = 3;
            122
        }),
        Err(DiscoveryFailure::ChangingResponse { .. })
    ));
    assert!(matches!(
        text::query(OP, |length, _| {
            *length = 3;
            0
        }),
        Err(DiscoveryFailure::Malformed {
            reason: MalformedReply::Size,
            ..
        })
    ));
}
