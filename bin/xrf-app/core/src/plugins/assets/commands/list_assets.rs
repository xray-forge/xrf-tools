use tauri::State;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayRoots};

use crate::TauriResult;
use crate::assets::AssetMountState;
use crate::execution::ExecutionState;

/// Every asset of one kind the roots hold, winner first and shadowed copies omitted; read on the pool, since a first
/// listing plans and mounts the roots.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_assets"))]
#[tauri::command(rename = "list_assets")]
pub async fn assets_list_assets(
  roots: XrayRoots,
  kind: XrayAssetType,
  state: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<Vec<XrayAsset>> {
  let state: AssetMountState = AssetMountState::clone(&state);

  execution
    .run_blocking("Listing assets", move || {
      let assets: Vec<XrayAsset> = state.search(&roots)?.list_assets(kind);

      log::info!("Listed {} assets of {kind:?} in the mounted roots", assets.len());

      Ok(assets)
    })
    .await?
}
