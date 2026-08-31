use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::state::EquipmentSpriteState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_sprite"))]
#[tauri::command(rename = "close_sprite")]
pub async fn sprite_equipment_close_sprite(state: State<'_, EquipmentSpriteState>) -> TauriResult {
  log::info!("Closing equipment file:");

  *state.system_ltx_path.lock().unwrap() = None;
  *state.equipment_sprite_path.lock().unwrap() = None;
  *state.equipment_sprite_name.lock().unwrap() = None;
  *state.equipment_descriptors.lock().unwrap() = None;
  *state.equipment_sprite_preview.lock().unwrap() = None;

  Ok(())
}
