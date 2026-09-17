use tauri::State;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayRoots};

use crate::core::assets::AssetMountState;
use crate::core::types::TauriResult;

/// Every asset of one kind the roots hold, winner first and shadowed copies omitted.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_assets"))]
#[tauri::command(rename = "list_assets")]
pub async fn assets_list_assets(
  roots: XrayRoots,
  kind: XrayAssetType,
  state: State<'_, AssetMountState>,
) -> TauriResult<Vec<XrayAsset>> {
  let assets: Vec<XrayAsset> = state.with_probe(&roots, |probe| probe.list_assets_of_type(kind))?;

  log::info!("Listed {} assets of {kind:?} in the mounted roots", assets.len());

  Ok(assets)
}
