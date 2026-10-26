use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;

/// What unpacking a whole archive project produced.
///
/// The two path fields are rendered for a person through `xrf_utils::format_path`, never addresses: a
/// host name that is not valid Unicode renders lossily rather than failing a run whose files are already
/// on disk. A caller that needs to open the destination uses the path it supplied.
///
/// Every count here describes what the run actually did, not what the project holds. That distinction only becomes
/// visible when a run stops early, which is exactly when a caller most needs the numbers to be true.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveUnpackResult {
  /// Volume files that were read, rendered for display.
  pub archives: Vec<String>,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  /// Root the files were written under, rendered for display.
  pub destination: String,
  /// Whether the run reached the end of its work or was stopped at an entry boundary.
  ///
  /// A cancelled run leaves what it had already written where it is: the files below `destination` are a real but
  /// partial tree, and nothing removes them. Read the counts below as what is on disk, never as a total.
  pub outcome: JobOutcome,
  /// Entries dealt with, directory rows included, which is what the counts are measured against.
  pub files_total: usize,
  /// Files actually written.
  pub files_unpacked: usize,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub prepare_duration: Duration,
  /// Bytes written, summed from the entries that were written rather than from the project.
  pub unpacked_size: u64,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub unpack_duration: Duration,
}
