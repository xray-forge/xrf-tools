use serde::Serialize;
use xrf_texture::EquipmentSlotOccupant;

use crate::plugins::sprite_equipment::location::EquipmentSheetLocation;
use crate::plugins::sprite_equipment::source::EquipmentSpriteOpen;

/// Everything the editor is told about one opened sheet.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSpriteMetadata {
  /// Name the sheet is streamed to the webview under.
  pub name: String,
  /// What this open was asked to read, so a reload repeats the request rather than a reconstruction of it.
  pub open: EquipmentSpriteOpen,
  /// Which copy of the sheet this is, and whether anything can write to it.
  pub location: EquipmentSheetLocation,
  /// Why the configuration was not read, when it was asked for and could not be.
  pub config_error: Option<String>,
  /// Every section occupying a slot on the sheet, in the order the configuration declares them.
  pub occupants: Vec<EquipmentSlotOccupant>,
}
