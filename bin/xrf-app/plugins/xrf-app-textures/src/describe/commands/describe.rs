use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_vfs::XrayRoots;

use crate::describe::{TextureDescription, TextureSource};

/// Describe one texture: its file, its descriptor as the engine reads it, and the pair the engine binds.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
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
      TextureDescription::describe_in(&assets, source, roots)
    })
    .await?
}
