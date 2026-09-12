use tauri::State;

use crate::core::session::SessionRestore;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteMetadata, EquipmentSpriteState};

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_sprite"))]
#[tauri::command(rename = "get_sprite")]
pub async fn sprite_equipment_get_sprite(
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<SessionRestore<EquipmentSpriteMetadata>> {
  Ok(SessionRestore::from(
    state
      .get()?
      .map(|opened| opened.map(|document| document.metadata.clone())),
  ))
}
