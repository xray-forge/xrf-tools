use tauri::State;

use crate::core::session::DocumentSessionId;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::state::EquipmentSpriteState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_sprite"))]
#[tauri::command(rename = "close_sprite")]
pub async fn sprite_equipment_close_sprite(
  session_ids: Vec<DocumentSessionId>,
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult {
  state.close(&session_ids)
}
