use tauri::State;
use xrf_dds::{DdsFile, DdsPng};
use xrf_ltx::Ltx;
use xrf_texture::InventorySpriteDescriptor;

use crate::core::error::error_to_string;
use crate::core::ltx_dialect_selection::select_ltx_dialect;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteMetadata, EquipmentSpriteState};

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_sprite"))]
#[tauri::command(rename = "open_sprite")]
pub async fn sprite_equipment_open_sprite(
  equipment_dds_path: &str,
  system_ltx_path: &str,
  is_dltx: bool,
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<EquipmentSpriteMetadata> {
  log::info!("Opening equipment file: {equipment_dds_path} - {system_ltx_path}");

  let name: &str = "equipment.png";

  let preview: DdsPng = DdsFile::read_from_path(equipment_dds_path)
    .and_then(|dds| dds.to_png())
    .map_err(|error| format!("Failed to open provided image file: {}", error))?;

  log::info!("Opened equipment dds file");

  let descriptors: Vec<InventorySpriteDescriptor> = InventorySpriteDescriptor::new_list_from_ltx(
    &Ltx::read_from_file_with_dialect(system_ltx_path, select_ltx_dialect(is_dltx).as_ref())
      .map_err(error_to_string)?,
  );

  let response = EquipmentSpriteMetadata {
    is_dltx,
    system_ltx_path: system_ltx_path.into(),
    path: equipment_dds_path.into(),
    name: name.into(),
    equipment_descriptors: descriptors.clone(),
  };

  *state.is_dltx.lock().unwrap() = is_dltx;
  *state.system_ltx_path.lock().unwrap() = Some(system_ltx_path.into());
  *state.equipment_sprite_name.lock().unwrap() = Some(name.into());
  *state.equipment_sprite_path.lock().unwrap() = Some(equipment_dds_path.into());
  *state.equipment_sprite_preview.lock().unwrap() = Some(preview.bytes);
  *state.equipment_descriptors.lock().unwrap() = Some(descriptors);

  Ok(response)
}
