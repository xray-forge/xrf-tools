use std::time::Duration;

use serde::Serialize;

use crate::progress_level::ProgressLevel;

/// One snapshot of a running job, as it crosses to whoever is watching.
///
/// The whole active stack rather than the deepest level, because a reader showing two bars needs both at the same
/// instant. Two snapshots taken a moment apart would let the outer bar describe a phase the inner one has left.
///
/// It carries no job identity and no wall-clock timestamp. Identity belongs to whoever addressed the job — repeating
/// it in every update would make the payload the second place it can be wrong — and elapsed time is measured from a
/// monotonic start, so a clock adjustment mid-run cannot make a job appear to run backwards.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
  /// The active stack, outermost first. Never empty while a job is reporting.
  pub levels: Vec<ProgressLevel>,
  /// How long the job has been running, preparation included.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  /// What the job is on right now, where saying so is meaningful.
  ///
  /// Replaced by the next snapshot and never accumulated: this is a line on screen, not a log. An operation running
  /// its units across a pool leaves it empty, because naming one arbitrary worker's entry reads as thrashing rather
  /// than as progress.
  pub detail: Option<String>,
}
