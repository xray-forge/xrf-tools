use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

use crate::EquipmentSpriteOpen;

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
