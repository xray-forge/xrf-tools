use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionOpening, SessionSnapshot};
use xrf_app_core::types::TauriResult;

use crate::document::EquipmentSpriteDocument;
use crate::metadata::EquipmentSpriteMetadata;
use crate::request::SpriteEquipmentOpenRequest;
use crate::state::EquipmentSpriteState;

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

  let opening: SessionOpening<EquipmentSpriteDocument> = state.begin_open(session_id)?;

  let assets: AssetMountState = AssetMountState::clone(&assets);
  let document = execution
    .run_blocking("Equipment sheet open", move || {
      EquipmentSpriteDocument::read(&assets, open)
    })
    .await??;

  Ok(opening.commit(document)?.map(|document| document.metadata.clone()))
}
