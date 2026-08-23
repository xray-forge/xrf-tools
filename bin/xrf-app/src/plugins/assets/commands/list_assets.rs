use tauri::State;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayWorldSpec};

use crate::core::assets::AssetWorldState;
use crate::core::types::TauriResult;

/// Every asset of one kind a world holds, winner first and shadowed copies omitted.
///
/// Flat rather than a tree, because a flat index is what a filter reads and what a tree is built from — the same
/// shape the archive explorer already builds its file tree out of. The kind is the caller's, so listing a new kind is
/// an argument rather than a command.
///
/// Assets keep the world's own logical paths, so an entry names the model a `visuals` open can then take verbatim.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_assets"))]
#[tauri::command(rename = "list_assets")]
pub async fn assets_list_assets(
  world: XrayWorldSpec,
  kind: XrayAssetType,
  state: State<'_, AssetWorldState>,
) -> TauriResult<Vec<XrayAsset>> {
  let assets: Vec<XrayAsset> = state.with_probe(&world, |probe| probe.list_assets_of_type(kind))?;

  log::info!("Listed {} assets of {kind:?} in the mounted world", assets.len());

  Ok(assets)
}
