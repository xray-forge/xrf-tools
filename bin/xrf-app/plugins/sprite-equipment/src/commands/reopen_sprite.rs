use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;

use crate::document::EquipmentSpriteDocument;
use crate::metadata::EquipmentSpriteMetadata;
use crate::source::EquipmentSpriteOpen;
use crate::state::EquipmentSpriteState;

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
