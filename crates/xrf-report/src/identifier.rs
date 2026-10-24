use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::Serialize;
use xrf_error::XrfError;

/// Error returned when a stable report identifier is empty.
#[derive(Debug, Eq, PartialEq)]
pub struct IdentifierError {
  kind: &'static str,
}

impl IdentifierError {
  fn empty(kind: &'static str) -> Self {
    Self { kind }
  }
}

impl Display for IdentifierError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{kind} cannot be empty", kind = self.kind)
  }
}

impl Error for IdentifierError {}

// Lets fallible identifier construction propagate with `?` where a caller returns `XrfResult`,
// instead of forcing an `expect` at every dynamic call site.
impl From<IdentifierError> for XrfError {
  fn from(value: IdentifierError) -> Self {
    Self::Invalid {
      message: value.to_string(),
    }
  }
}

macro_rules! identifier {
  ($name:ident, $kind:literal) => {
    #[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
    #[serde(transparent)]
    pub struct $name(String);

    impl $name {
      /// Creates a stable identifier.
      ///
      /// # Errors
      ///
      /// Returns [`IdentifierError`] when `value` is empty.
      pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value: String = value.into();

        if value.is_empty() {
          Err(IdentifierError::empty($kind))
        } else {
          Ok(Self(value))
        }
      }

      pub fn as_str(&self) -> &str {
        &self.0
      }
    }

    impl Display for $name {
      fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
      }
    }
  };
}

identifier!(CheckId, "check ID");
identifier!(RuleId, "rule ID");

#[cfg(test)]
mod tests {
  use super::{CheckId, RuleId};

  #[test]
  fn rejects_empty_identifiers() {
    assert_eq!(CheckId::new("").unwrap_err().to_string(), "check ID cannot be empty");
    assert_eq!(RuleId::new("").unwrap_err().to_string(), "rule ID cannot be empty");
  }
}
