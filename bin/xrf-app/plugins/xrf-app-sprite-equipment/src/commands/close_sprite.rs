use tauri::State;
use xrf_app_core::session::SessionId;

use crate::EquipmentSprites;

/// Close the sheets `session_ids` name, committed or opening; a closed one answers no read.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_sprite"))]
#[tauri::command(rename = "close_sprite")]
pub fn sprite_equipment_close_sprite(sprites: State<'_, EquipmentSprites>, session_ids: Vec<SessionId>) {
  sprites.close(&session_ids);
}
