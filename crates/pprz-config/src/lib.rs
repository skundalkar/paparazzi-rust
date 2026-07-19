//! Configuration-domain types for Paparazzi-compatible tools.

/// Identifies an airframe configuration without interpreting its XML yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AirframeId(String);

impl AirframeId {
    /// Validates and creates a non-empty airframe identifier.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::EmptyAirframeId`] when `value` contains no
    /// visible characters.
    pub fn new(value: impl Into<String>) -> Result<Self, ConfigError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ConfigError::EmptyAirframeId);
        }
        Ok(Self(value))
    }

    /// Returns the identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Errors raised while validating configuration values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// An airframe identifier contained no visible characters.
    EmptyAirframeId,
}

#[cfg(test)]
mod tests {
    use super::{AirframeId, ConfigError};

    #[test]
    fn rejects_empty_identifier() {
        assert_eq!(AirframeId::new("  "), Err(ConfigError::EmptyAirframeId));
    }
}
