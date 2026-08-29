use std::time::Duration;

use serde::Serialize;

/// What unpacking a whole archive project produced.
///
/// The two path fields are rendered for a person through `xrf_utils::format_path`, never addresses: a
/// host name that is not valid Unicode renders lossily rather than failing a run whose files are already
/// on disk. A caller that needs to open the destination uses the path it supplied.
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
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub prepare_duration: Duration,
  pub unpacked_size: u64,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub unpack_duration: Duration,
}
