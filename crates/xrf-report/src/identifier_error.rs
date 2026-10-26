use std::error::Error;
use std::fmt::{Display, Formatter};

use xrf_error::XrfError;

/// Error returned when a stable report identifier is empty.
#[derive(Debug, Eq, PartialEq)]
pub struct IdentifierError {
  kind: &'static str,
}

impl IdentifierError {
  pub(crate) fn empty(kind: &'static str) -> Self {
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
