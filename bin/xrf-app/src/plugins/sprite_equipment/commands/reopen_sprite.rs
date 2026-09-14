use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::document::EquipmentSpriteDocument;
use crate::plugins::sprite_equipment::metadata::EquipmentSpriteMetadata;
use crate::plugins::sprite_equipment::source::EquipmentSpriteOpen;
use crate::plugins::sprite_equipment::state::EquipmentSpriteState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "reopen_sprite"))]
#[tauri::command(rename = "reopen_sprite")]
pub async fn sprite_equipment_reopen_sprite(
  session_id: SessionId,
  opening_id: SessionId,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<SessionSnapshot<EquipmentSpriteMetadata>> {
  let open: EquipmentSpriteOpen = state.begin_reload(opening_id, session_id)?.metadata.open.clone();

  let assets: AssetMountState = AssetMountState::clone(&assets);
  let document = execution
    .run_blocking("Equipment sheet reload", move || {
      EquipmentSpriteDocument::read(&assets, open)
    })
    .await??;

  Ok(
    state
      .commit_open(opening_id, document)?
      .map(|document| document.metadata.clone()),
  )
}
