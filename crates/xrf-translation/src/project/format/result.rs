use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;

/// What one formatting run judged and changed.
///
/// Field for field what `LtxProjectFormatResult` reports, so the two `format --check` commands answer a consumer in one
/// shape. Its own type rather than that one: a crate deposits its own result, and reaching into `xrf-ltx` for a shape
/// would tie a translation run's wire contract to an unrelated format's.
///
/// No findings array, unlike this crate's parse and verify results. A formatter's answer is the list of files to fix,
/// and a finding would spend a subject, a rule id and a message carrying one bit each over a single rule.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFormatResult {
  /// Whether the run reached the end of the set or was stopped between files.
  ///
  /// A stopped rewrite leaves the files it had already formatted formatted and the rest untouched, which is a state
  /// running it again resolves. Nothing is removed and nothing is half-written.
  pub outcome: JobOutcome,
  /// Everything the run took, measured from when its caller created the job handle.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  /// How much of `duration` had already passed when the per-file work began.
  ///
  /// Walking the paths and selecting sources out of them happens before a single file is read, and on a cold filesystem
  /// or a large tree that is most of the wait. Named rather than folded away, so the split stays readable.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub startup_duration: Duration,
  /// Sources that were not canonical: rewritten by a normal run, reported by a check.
  pub invalid_files: usize,
  /// Which ones, in selection order.
  pub to_format: Vec<PathBuf>,
  pub total_files: usize,
  pub valid_files: usize,
}

impl ProjectFormatResult {
  pub fn new() -> Self {
    Self {
      outcome: JobOutcome::Completed,
      duration: Duration::ZERO,
      startup_duration: Duration::ZERO,
      invalid_files: 0,
      to_format: Vec::new(),
      total_files: 0,
      valid_files: 0,
    }
  }

  /// Record the verdict for one source.
  pub(crate) fn record(&mut self, path: PathBuf, is_changed: bool) {
    if is_changed {
      self.invalid_files += 1;
      self.to_format.push(path);
    } else {
      self.valid_files += 1;
    }

    self.total_files += 1;
  }
}
