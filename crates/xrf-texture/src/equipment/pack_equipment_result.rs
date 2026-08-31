use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;

#[derive(Debug, Default, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct PackEquipmentResult {
  /// Whether the run drew every section or was stopped between them.
  ///
  /// The sheet is one image written once at the end, so a stopped run leaves nothing behind: the counts describe what
  /// it had drawn in memory, and no file was replaced.
  pub outcome: JobOutcome,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  pub saved_at: PathBuf,
  pub saved_width: u32,
  pub saved_height: u32,
  pub packed_count: u32,
  pub skipped_count: u32,
}
