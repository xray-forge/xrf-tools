use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::document::EquipmentSpriteDocument;
use crate::plugins::sprite_equipment::metadata::EquipmentSpriteMetadata;
use crate::plugins::sprite_equipment::request::SpriteEquipmentOpenRequest;
use crate::plugins::sprite_equipment::state::EquipmentSpriteState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_sprite"))]
#[tauri::command(rename = "open_sprite")]
pub async fn sprite_equipment_open_sprite(
  request: SpriteEquipmentOpenRequest,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  state: State<'_, EquipmentSpriteState>,
) -> TauriResult<SessionSnapshot<EquipmentSpriteMetadata>> {
  let SpriteEquipmentOpenRequest { session_id, open } = request;

  log::info!("Opening equipment sheet: {}", open.sheet.label());

  state.begin_open(session_id)?;

  let assets: AssetMountState = AssetMountState::clone(&assets);
  let document = execution
    .run_blocking("Equipment sheet open", move || {
      EquipmentSpriteDocument::read(&assets, open)
    })
    .await??;

  Ok(
    state
      .commit_open(session_id, document)?
      .map(|document| document.metadata.clone()),
  )
}
