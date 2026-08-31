use std::sync::MutexGuard;

use tauri::State;
use xrf_dds::{DdsFile, DdsPng};
use xrf_ltx::Ltx;
use xrf_texture::InventorySpriteDescriptor;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteMetadata, EquipmentSpriteState};

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "reopen_sprite"))]
#[tauri::command(rename = "reopen_sprite")]
pub async fn sprite_equipment_reopen_sprite(
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<EquipmentSpriteMetadata> {
  log::info!("Reopening equipment sprite");

  let ltx_path_lock: MutexGuard<Option<String>> = state.system_ltx_path.as_ref().lock().unwrap();
  let dds_path_lock: MutexGuard<Option<String>> = state.equipment_sprite_path.lock().unwrap();
  let dds_name_lock: MutexGuard<Option<String>> = state.equipment_sprite_name.as_ref().lock().unwrap();

  if ltx_path_lock.is_none() || dds_path_lock.is_none() || dds_name_lock.is_none() {
    return Err(String::from(
      "Failed to reopen equipment sprites - no active sprite open now",
    ));
  }

  let dds_name: &String = dds_name_lock.as_ref().unwrap();
  let ltx_path: &String = ltx_path_lock.as_ref().unwrap();
  let dds_path: &String = dds_path_lock.as_ref().unwrap();

  let preview: DdsPng = DdsFile::read_from_path(dds_path)
    .and_then(|dds| dds.to_png())
    .map_err(|error| format!("Failed to open provided image file: {}", error))?;

  let descriptors: Vec<InventorySpriteDescriptor> =
    InventorySpriteDescriptor::new_list_from_ltx(&Ltx::read_from_file_full(ltx_path).map_err(error_to_string)?);

  let response = EquipmentSpriteMetadata {
    system_ltx_path: ltx_path.into(),
    path: dds_path.into(),
    name: dds_name.into(),
    equipment_descriptors: descriptors.clone(),
  };

  *state.equipment_sprite_preview.lock().unwrap() = Some(preview.bytes);
  *state.equipment_descriptors.lock().unwrap() = Some(descriptors);

  Ok(response)
}
