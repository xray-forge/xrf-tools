use serde::{Deserialize, Serialize};
use xrf_pack::ArchivePatchConfig;

/// Shared request for archive comparison and patch publication.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesPatchRequest {
  /// What to compare and where to publish the difference.
  pub config: ArchivePatchConfig,
  /// Whether an existing output may be overwritten. Ignored by a comparison, which writes nothing.
  pub is_forced: bool,
  /// Whether a checksum match should be proven by comparing the payloads themselves.
  pub is_verifying_payload: bool,
}
