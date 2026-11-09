#[cfg(test)]
use std::os::windows::ffi::OsStringExt;
use std::{
    ffi::{OsStr, OsString},
    io,
    os::windows::ffi::OsStrExt,
    path::Path,
};
/// CRT argv quoting, including trailing backslashes and embedded quotes.
fn quote(value: &OsStr) -> Vec<u16> {
    let mut output = vec![34];
    let mut slashes = 0;
    for character in value.encode_wide() {
        if character == 92 {
            slashes += 1;
            continue;
        }
        output.extend(std::iter::repeat_n(
            92,
            if character == 34 {
                slashes * 2 + 1
            } else {
                slashes
            },
        ));
        output.push(character);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n(92, slashes * 2));
    output.push(34);
    output
}
pub(super) fn arguments(executable: &Path, args: &[OsString]) -> io::Result<Vec<u16>> {
    let mut result = quote(executable.as_os_str());
    for argument in args {
        result.push(32);
        result.extend(quote(argument));
    }
    if result.contains(&0) || result.len() >= 32767 {
        return Err(io::Error::other("NUL or oversized GUI command line"));
    }
    result.push(0);
    Ok(result)
}
#[test]
fn quoting_preserves_unicode_quotes_and_trailing_backslashes() {
    assert_eq!(
        OsString::from_wide(&quote(OsStr::new("a b\\"))),
        OsString::from("\"a b\\\\\"")
    );
    assert_eq!(
        OsString::from_wide(&quote(OsStr::new("a\"β"))),
        OsString::from("\"a\\\"β\"")
    );
}
