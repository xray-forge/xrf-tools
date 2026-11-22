use std::fmt::{Display, Formatter};

use serde::Serialize;

/// Whether a finding fails its check, or is reported and fails it only when the run is strict.
///
/// The producer that knows what the defect costs decides, once, at the finding: a reader of a report never has to know
/// a check's rules to tell what failed it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
  /// What the consumer cannot work with: the engine asserts on it or refuses it, or the file does not read.
  Failure,
  /// What the consumer tolerates: it works around it, shows something less, or never reaches it.
  Warning,
}

impl Display for Severity {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(match self {
      Self::Failure => "failure",
      Self::Warning => "warning",
    })
  }
}
