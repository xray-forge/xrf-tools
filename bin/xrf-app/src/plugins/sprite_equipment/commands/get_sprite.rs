use std::sync::MutexGuard;

use tauri::State;
use xrf_texture::InventorySpriteDescriptor;

use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteMetadata, EquipmentSpriteState};

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_sprite"))]
#[tauri::command(rename = "get_sprite")]
pub async fn sprite_equipment_get_sprite(
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<Option<EquipmentSpriteMetadata>> {
  log::debug!("Getting equipment sprite");

  let ltx_path_lock: MutexGuard<Option<String>> = state.system_ltx_path.as_ref().lock().unwrap();
  let dds_path_lock: MutexGuard<Option<String>> = state.equipment_sprite_path.lock().unwrap();
  let name_lock: MutexGuard<Option<String>> = state.equipment_sprite_name.lock().unwrap();
  let equipment_lock: MutexGuard<Option<Vec<InventorySpriteDescriptor>>> = state.equipment_descriptors.lock().unwrap();

  if ltx_path_lock.is_none() || equipment_lock.is_none() || name_lock.is_none() {
    return Ok(None);
  }

  Ok(Some(EquipmentSpriteMetadata {
    is_dltx: *state.is_dltx.lock().unwrap(),
    system_ltx_path: ltx_path_lock.as_ref().unwrap().clone(),
    path: dds_path_lock.as_ref().unwrap().clone(),
    name: name_lock.as_ref().unwrap().clone(),
    equipment_descriptors: equipment_lock.as_ref().unwrap().clone(),
  }))
}
