use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::{EquipmentSpriteMetadata, EquipmentSprites};

/// The sheet `tool` holds in the asking window, the newest it opened, for a page attaching to it after a reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_sprite")]
pub async fn sprite_equipment_restore_sprite(
  sprites: State<'_, EquipmentSprites>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<EquipmentSpriteMetadata>> {
  Ok(
    sprites
      .restore_as(&SessionScope::of(&window, tool), |document| document.metadata.clone())
      .into(),
  )
}
