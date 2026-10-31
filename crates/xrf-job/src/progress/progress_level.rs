use serde::Serialize;

use crate::progress::ProgressUnit;

/// One level of an active job's progress, as reported.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressLevel {
  /// Stable machine identity, declared as a constant beside the operation that enters it, and what a reader shows.
  pub id: String,
  pub completed: u64,
  /// Absent where the work cannot be counted before it is done.
  pub total: Option<u64>,
  pub unit: ProgressUnit,
}
