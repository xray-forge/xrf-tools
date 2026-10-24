use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;

use crate::request::SpriteEquipmentOpenRequest;
use crate::{EquipmentSpriteDocument, EquipmentSpriteMetadata, EquipmentSprites};

/// Open an equipment sheet with the configuration naming what sits on it, read on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_sprite"))]
#[tauri::command(rename = "open_sprite")]
pub async fn sprite_equipment_open_sprite(
  request: SpriteEquipmentOpenRequest,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  sprites: State<'_, EquipmentSprites>,
) -> TauriResult<SessionSnapshot<EquipmentSpriteMetadata>> {
  let SpriteEquipmentOpenRequest { session_id, open } = request;

  log::info!("Opening equipment sheet: {}", open.sheet.label());

  let assets: AssetMountState = AssetMountState::clone(&assets);
  let opened = sprites
    .open_on_pool(&execution, session_id, "Opening an equipment sheet", move || {
      EquipmentSpriteDocument::read(&assets, open)
    })
    .await?;

  Ok(opened.map(|document| document.metadata.clone()))
}
