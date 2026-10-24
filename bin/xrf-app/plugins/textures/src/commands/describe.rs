use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::types::TauriResult;
use xrf_vfs::XrayRoots;

use crate::description::TextureDescription;
use crate::source::TextureSource;

/// Describe one texture: its file, its descriptor as the engine reads it, and the pair the engine binds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe"))]
#[tauri::command(rename = "describe")]
pub async fn textures_describe(
  source: TextureSource,
  roots: XrayRoots,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TextureDescription> {
  log::info!("Describing texture: {}", source.label());

  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Describing texture", move || {
      let roots: XrayRoots = roots.centred_on(source.physical_path());

      TextureDescription::describe(&assets.search(&roots)?, source, roots.clone())
    })
    .await?
}
