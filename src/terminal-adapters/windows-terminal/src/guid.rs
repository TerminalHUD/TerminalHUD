use crate::Error;
use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Guid(String);
impl Guid {
    pub fn parse(value: &str) -> Result<Self, Error> {
        let bare = value
            .strip_prefix('{')
            .and_then(|v| v.strip_suffix('}'))
            .unwrap_or(value);
        if bare.len() != 36
            || !bare.bytes().enumerate().all(|(index, byte)| {
                if [8, 13, 18, 23].contains(&index) {
                    byte == b'-'
                } else {
                    byte.is_ascii_hexdigit()
                }
            })
        {
            return Err(Error::Invalid("invalid GUID"));
        }
        Ok(Self(format!("{{{}}}", bare.to_ascii_lowercase())))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn bare(&self) -> &str {
        &self.0[1..37]
    }
}
impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
