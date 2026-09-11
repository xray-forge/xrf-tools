use xrf_dds::{DdsFile, DdsPng};
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::Ltx;
use xrf_texture::InventorySpriteDescriptor;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteDocument, EquipmentSpriteMetadata};

/// Reads every part before any of it can become the opened sprite.
pub fn read_sprite(dds_path: &str, ltx_path: &str, is_dltx: bool) -> TauriResult<EquipmentSpriteDocument> {
  let preview: DdsPng = DdsFile::read_from_path(dds_path)
    .and_then(|dds| dds.to_png())
    .map_err(error_to_string)?;
  let descriptors: Vec<InventorySpriteDescriptor> = InventorySpriteDescriptor::new_list_from_ltx(
    &Ltx::read_from_file_with_dialect(ltx_path, select_ltx_dialect(is_dltx).as_ref()).map_err(error_to_string)?,
  );

  Ok(EquipmentSpriteDocument {
    metadata: EquipmentSpriteMetadata {
      is_dltx,
      system_ltx_path: ltx_path.into(),
      path: dds_path.into(),
      name: String::from("equipment.png"),
      equipment_descriptors: descriptors,
    },
    preview: preview.bytes,
  })
}
