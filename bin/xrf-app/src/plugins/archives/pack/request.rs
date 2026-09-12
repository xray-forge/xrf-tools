use serde::{Deserialize, Serialize};
use xrf_pack::ArchivePackConfig;

/// Archive packing request.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesPackRequest {
  /// What to pack and how.
  pub config: ArchivePackConfig,
  /// Whether an existing output may be overwritten.
  pub is_forced: bool,
}
