use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::{EquipmentSpriteDocument, EquipmentSpriteMetadata, EquipmentSprites};

/// Read the sheet `previous_id` holds again, as it was asked for, into the new handle `session_id`; the previous handle
/// is left as it is, for the page to close once the reload is shown, and a failed reload leaves it untouched.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "reload_sprite"))]
#[tauri::command(rename = "reload_sprite")]
pub async fn sprite_equipment_reload_sprite(
  session_id: SessionId,
  previous_id: SessionId,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  sprites: State<'_, EquipmentSprites>,
) -> TauriResult<SessionSnapshot<EquipmentSpriteMetadata>> {
  let reloaded = EquipmentSpriteDocument::reload(&sprites, &execution, &assets, session_id, previous_id).await?;

  Ok(reloaded.map(|document| document.metadata.clone()))
}
