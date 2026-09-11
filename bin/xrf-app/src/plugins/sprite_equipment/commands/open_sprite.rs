use tauri::State;

use crate::core::session::DocumentSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::document::read_sprite;
use crate::plugins::sprite_equipment::request::SpriteEquipmentOpenRequest;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteMetadata, EquipmentSpriteState};

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_sprite"))]
#[tauri::command(rename = "open_sprite")]
pub async fn sprite_equipment_open_sprite(
  request: SpriteEquipmentOpenRequest,
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<DocumentSnapshot<EquipmentSpriteMetadata>> {
  let SpriteEquipmentOpenRequest {
    session_id,
    equipment_dds_path,
    system_ltx_path,
    is_dltx,
  } = request;

  state.begin_open(session_id)?;

  let document = read_sprite(&equipment_dds_path, &system_ltx_path, is_dltx)?;

  Ok(
    state
      .commit_open(session_id, document)?
      .map(|document| document.metadata.clone()),
  )
}
