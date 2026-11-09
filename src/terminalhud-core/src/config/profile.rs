use super::ValidationError;

/// Opaque portable identity; resolution belongs to an adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileId(String);

impl ProfileId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value == "auto"
            || value.trim().is_empty()
            || value.len() > 256
            || value.chars().any(char::is_control)
        {
            return Err(ValidationError::new(
                "terminal.defaultHudProfile",
                "profile ID must not be reserved auto and must contain 1..=256 UTF-8 bytes, non-whitespace, and no control characters",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ProfileSelection {
    #[default]
    Auto,
    Id(ProfileId),
}
