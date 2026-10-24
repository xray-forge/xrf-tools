use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Source and output paths of a standalone spawn conversion.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnConversionRequest {
  pub source: PathBuf,
  pub destination: PathBuf,
}
