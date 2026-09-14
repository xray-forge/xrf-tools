use serde::Serialize;
use xrf_texture::EquipmentSlotOccupant;

use crate::core::session::Session;

/// Metadata and bytes belong to one immutable publication.
pub struct EquipmentSpriteDocument {
  pub metadata: EquipmentSpriteMetadata,
  pub preview: Vec<u8>,
}

pub type EquipmentSpriteState = Session<EquipmentSpriteDocument>;

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSpriteMetadata {
  pub path: String,
  pub name: String,
  pub system_ltx_path: String,
  /// Whether these occupants came out of a DLTX-resolved config tree.
  pub is_dltx: bool,
  /// Every section occupying a slot on the sheet, in the order the configuration declares them.
  pub occupants: Vec<EquipmentSlotOccupant>,
}
