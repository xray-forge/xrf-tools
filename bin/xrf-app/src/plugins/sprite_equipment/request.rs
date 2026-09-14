use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::session::SessionId;
use crate::plugins::sprite_equipment::source::EquipmentSpriteOpen;

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

/// The complete identity and inputs of one sprite opening.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpriteEquipmentOpenRequest {
  pub session_id: SessionId,
  /// What to read, which the opened document keeps so a reload can repeat it.
  #[serde(flatten)]
  pub open: EquipmentSpriteOpen,
}
