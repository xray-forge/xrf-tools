use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::EquipmentSpriteState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_sprite"))]
#[tauri::command(rename = "close_sprite")]
pub fn sprite_equipment_close_sprite(
  session_ids: Vec<SessionId>,
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult {
  state.close(&session_ids)
}
