use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_pack::{ArchivePackConfig, ArchivePatchConfig};

use crate::core::session::DocumentSessionId;

/// Archive unpacking request.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesUnpackRequest {
  /// Archive or directory of archives to read.
  pub from: PathBuf,
  /// Directory to write the contents into.
  pub destination: PathBuf,
}

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

/// Directory extraction request for an open archive project.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesExtractRequest {
  pub session_id: DocumentSessionId,
  /// Directory inside the archive to extract.
  pub prefix: String,
  /// Directory to write the contents into.
  pub destination: PathBuf,
}
