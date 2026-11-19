use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Source and output paths of a standalone spawn conversion.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnConvertRequest {
  pub source: PathBuf,
  pub destination: PathBuf,
  /// Whether the output may replace a file or directory already at `destination`, in any letter case; unforced, the
  /// conversion refuses before reading.
  pub is_forced: bool,
}
