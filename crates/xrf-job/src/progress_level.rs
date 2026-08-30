use serde::Serialize;

use crate::progress_unit::ProgressUnit;

/// One level of an active job's progress, as reported.
///
/// A level counts whatever it is made of: a parent counts its finished children, a leaf counts its own units. That is
/// what lets a run reporting `["verify" 2/7, "textures" 400/40000]` and one reporting
/// `["unpack" 1/2, "write" 45000/100000]` use one mechanism instead of a phase concept and a unit concept.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressLevel {
  /// Stable machine identity, declared as a constant beside the operation that enters it.
  ///
  /// Separate from `label` because a reader keying on a phase must not be keying on wording somebody will improve.
  pub id: String,
  /// What to call this level in front of a person, where the id is not already presentable.
  pub label: Option<String>,
  pub completed: u64,
  /// Absent where the work cannot be counted before it is done.
  ///
  /// Honest rather than convenient: a reader shows an indeterminate state and the active phase, which is true, instead
  /// of a percentage derived from a total nobody knows.
  pub total: Option<u64>,
  pub unit: ProgressUnit,
}
