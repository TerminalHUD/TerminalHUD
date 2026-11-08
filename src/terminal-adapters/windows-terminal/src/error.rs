use std::fmt;
#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Settings(String),
    MissingProfile,
    NoMatchingWindow,
    UnresolvedSource,
    ElevatedProfile,
    Collision,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
