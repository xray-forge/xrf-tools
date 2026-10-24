use tauri::State;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::types::TauriResult;

use crate::metadata::EquipmentSpriteMetadata;
use crate::state::EquipmentSpriteState;

/// What sheet a tool holds, the newest it opened, for a page attaching to it after a reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_sprite"))]
#[tauri::command(rename = "get_sprite")]
pub async fn sprite_equipment_get_sprite(
  state: State<'_, EquipmentSpriteState>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<EquipmentSpriteMetadata>> {
  Ok(SessionRestore::from(
    state
      .restore(&scope)?
      .map(|opened| opened.map(|document| document.metadata.clone())),
  ))
}
