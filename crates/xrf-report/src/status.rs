use std::fmt::{Display, Formatter};

use serde::Serialize;

/// The outcome of a completed check.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
  Error,
  Failed,
  Incomplete,
  Passed,
  #[default]
  Skipped,
}

impl Status {
  /// Aggregates check outcomes by severity.
  pub fn aggregate(statuses: impl IntoIterator<Item = Self>) -> Self {
    statuses
      .into_iter()
      .min_by_key(|status| status.severity())
      .unwrap_or(Self::Skipped)
  }

  pub const fn from_is_valid(is_valid: bool) -> Self {
    if is_valid { Self::Passed } else { Self::Failed }
  }

  const fn severity(self) -> u8 {
    match self {
      Self::Error => 0,
      Self::Failed => 1,
      Self::Incomplete => 2,
      Self::Passed => 3,
      Self::Skipped => 4,
    }
  }
}

impl Display for Status {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    let value: &str = match self {
      Self::Error => "error",
      Self::Failed => "failed",
      Self::Incomplete => "incomplete",
      Self::Passed => "passed",
      Self::Skipped => "skipped",
    };

    formatter.write_str(value)
  }
}

#[cfg(test)]
mod tests {
  use super::Status;

  #[test]
  fn aggregates_statuses_by_severity() {
    assert_eq!(Status::aggregate([]), Status::Skipped);
    assert_eq!(
      Status::aggregate([Status::Passed, Status::Incomplete]),
      Status::Incomplete
    );
    assert_eq!(Status::aggregate([Status::Failed, Status::Error]), Status::Error);
    assert_eq!(
      Status::aggregate([
        Status::Skipped,
        Status::Passed,
        Status::Incomplete,
        Status::Failed,
        Status::Error,
      ]),
      Status::Error
    );
  }
}
