use tauri::State;

use crate::core::session::{SessionRestore, SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::metadata::EquipmentSpriteMetadata;
use crate::plugins::sprite_equipment::state::EquipmentSpriteState;

/// What sheet a tool holds, the newest it opened, for a page attaching to it after a reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_sprite"))]
#[tauri::command(rename = "get_sprite")]
pub async fn sprite_equipment_get_sprite(
  state: State<'_, EquipmentSpriteState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<EquipmentSpriteMetadata>> {
  Ok(SessionRestore::from(
    state
      .restore(&scopes.list(&scope))?
      .map(|opened| opened.map(|document| document.metadata.clone())),
  ))
}
