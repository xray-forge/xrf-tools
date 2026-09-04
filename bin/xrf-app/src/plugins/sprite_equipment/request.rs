use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// What an equipment sprite pack was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct PackSpriteRequest {
  /// Directory of loose icons to draw from.
  pub source_path: PathBuf,
  /// Sheet to write.
  pub output_path: PathBuf,
  /// `system.ltx` declaring which icons exist and where they sit.
  pub system_ltx_path: PathBuf,
  /// Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
}
