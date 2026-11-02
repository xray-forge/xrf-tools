use std::time::Duration;

use xrf_job::JobOutcome;

/// What one tree write landed on disk.
pub(crate) struct TreeWriteSummary {
  /// Files written, which leaves out directory records.
  pub(crate) files: usize,
  /// Payload bytes written across those files.
  pub(crate) bytes: u64,
  /// Whether every entry was reached or the run was stopped between entries.
  pub(crate) outcome: JobOutcome,
  /// Job time at which every directory existed and the first payload could be written.
  pub(crate) prepared_at: Duration,
}
