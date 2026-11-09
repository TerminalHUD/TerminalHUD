use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError {
    pub property: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn new(property: &'static str, message: impl Into<String>) -> Self {
        Self {
            property,
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.property, self.message)
    }
}

impl std::error::Error for ValidationError {}
