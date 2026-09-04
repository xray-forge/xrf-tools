use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_pack::ArchivePackConfig;

/// What an archive unpack was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesUnpackRequest {
  /// Archive or directory of archives to read.
  pub from: PathBuf,
  /// Directory to write the contents into.
  pub destination: PathBuf,
}

/// What an archive pack was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesPackRequest {
  /// What to pack and how.
  pub config: ArchivePackConfig,
  /// Whether an existing output may be overwritten.
  pub is_forced: bool,
}

/// What an extraction out of an open archive project was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesExtractRequest {
  /// Directory inside the archive to extract.
  pub prefix: String,
  /// Directory to write the contents into.
  pub destination: PathBuf,
}
