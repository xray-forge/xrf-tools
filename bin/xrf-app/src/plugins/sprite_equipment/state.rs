use serde::Serialize;
use xrf_texture::InventorySpriteDescriptor;

use crate::core::session::DocumentSession;

/// Metadata and bytes belong to one immutable publication.
pub struct EquipmentSpriteDocument {
  pub metadata: EquipmentSpriteMetadata,
  pub preview: Vec<u8>,
}

pub type EquipmentSpriteState = DocumentSession<EquipmentSpriteDocument>;

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSpriteMetadata {
  pub path: String,
  pub name: String,
  pub system_ltx_path: String,
  /// Whether these descriptors came out of a DLTX-resolved config tree.
  pub is_dltx: bool,
  pub equipment_descriptors: Vec<InventorySpriteDescriptor>,
}
