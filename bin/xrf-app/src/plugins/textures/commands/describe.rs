use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::textures::description::TextureDescription;
use crate::plugins::textures::source::TextureSource;

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

      assets.with_probe(&roots, |probe| {
        TextureDescription::describe(probe, source, roots.clone())
      })?
    })
    .await?
}
