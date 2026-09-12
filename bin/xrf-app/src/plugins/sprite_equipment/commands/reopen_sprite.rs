use tauri::State;

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::document::read_sprite;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteMetadata, EquipmentSpriteState};

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "reopen_sprite"))]
#[tauri::command(rename = "reopen_sprite")]
pub async fn sprite_equipment_reopen_sprite(
  session_id: SessionId,
  opening_id: SessionId,
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<SessionSnapshot<EquipmentSpriteMetadata>> {
  let previous = state.begin_reload(opening_id, session_id)?;
  let metadata = &previous.metadata;
  let document = read_sprite(&metadata.path, &metadata.system_ltx_path, metadata.is_dltx)?;

  Ok(
    state
      .commit_open(opening_id, document)?
      .map(|document| document.metadata.clone()),
  )
}
