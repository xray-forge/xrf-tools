use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;

/// What one packing run produced.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePackResult {
  /// Volumes written, in mount order.
  ///
  /// A volume appears here once it has been closed, so on a run that stopped early this is the part of the set that
  /// is structurally complete — not the part that is usable, since a set missing its later volumes is missing entries.
  pub volumes: Vec<PathBuf>,
  /// Every volume path this run created, closed or not.
  ///
  /// Wider than `volumes` on purpose. A volume is opened with `File::create`, so it exists — and has replaced whatever
  /// stood at that path — from the moment writing begins. A run that stopped leaves the last one unfinished and absent
  /// from `volumes`, and a caller telling the user what is now on disk needs the paths rather than the successes.
  pub volumes_opened: Vec<PathBuf>,
  /// Whether the run reached the end of its work or was stopped between entries.
  ///
  /// A cancelled pack leaves an unusable partial volume set behind and removes nothing: the paths it opened may well
  /// have held a working archive set before it started, and nothing here can tell what this run created from what it
  /// overwrote. Report `volumes_opened` to whoever has to clean up.
  pub outcome: JobOutcome,
  pub files_total: usize,
  /// Files the include, exclude, and skip rules left out.
  pub files_skipped: usize,
  pub files_stored: usize,
  pub files_compressed: usize,
  /// Files that shared an identical earlier payload and cost only a descriptor row.
  pub files_aliased: usize,
  pub size_source: u64,
  pub size_written: u64,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
}
