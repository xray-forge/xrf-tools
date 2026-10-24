use tauri::State;
use xrf_engine_target::XrayEngineResolution;
use xrf_vfs::XrayRoots;

use crate::TauriResult;
use crate::assets::AssetMountState;
use crate::execution::ExecutionState;

/// Detect which engine roots target, as an open left to detection would read them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "detect_engine"))]
#[tauri::command(rename = "detect_engine")]
pub async fn assets_detect_engine(
  roots: XrayRoots,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<XrayEngineResolution> {
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Detecting the engine", move || {
      let resolution: XrayEngineResolution = assets.detect_engine(&roots);

      log::info!("Detected roots {} as {}", roots.describe(), resolution.describe());

      resolution
    })
    .await
}
