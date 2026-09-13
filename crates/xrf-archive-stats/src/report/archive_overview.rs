use serde::Serialize;

use crate::report::archive_measure::ArchiveMeasure;

/// What a subject holds, before any breakdown of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOverview {
  /// Files and the bytes they hold, which is the total every breakdown sums back to.
  pub total: ArchiveMeasure,
  /// Sources answered from: volumes for a volume set, mounts for a world.
  pub sources: u64,
  /// Entries naming a directory rather than a file.
  pub directories: u64,
  /// Files of zero length.
  pub empty_files: u64,
  /// Unpacked bytes of the largest single file.
  pub largest_file: u64,
  /// Mean unpacked bytes per file, zero when there are none.
  pub mean_file: u64,
  /// Median unpacked bytes per file, zero when there are none.
  pub median_file: u64,
}
