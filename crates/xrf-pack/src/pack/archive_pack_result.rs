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
  /// A volume appears here once it has been closed. On a forced run that stopped early this is the part of the set
  /// that is structurally complete — not the part that is usable, since a set missing its later volumes is missing
  /// entries. On any other run that did not finish it is empty, because such a run publishes nothing.
  pub volumes: Vec<PathBuf>,
  /// Every volume path this run created, closed or not.
  ///
  /// Wider than `volumes` on purpose. A volume is opened with `File::create`, so it exists — and has replaced whatever
  /// stood at that path — from the moment writing begins.
  ///
  /// Empty on a run that did not finish and was not forced: such a run began over a destination holding no volume of
  /// its set, so every file it made was its own and was removed again. A forced run is where this earns its place —
  /// there the same paths may have held a working set beforehand, deleting them would compound the loss, and the
  /// caller needs the list to say what is now on disk.
  pub volumes_opened: Vec<PathBuf>,
  /// Whether the run reached the end of its work or was stopped between entries.
  ///
  /// A cancelled pack publishes nothing and leaves the destination as it found it, unless it was forced — see
  /// `volumes_opened` for what a forced run leaves behind.
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
