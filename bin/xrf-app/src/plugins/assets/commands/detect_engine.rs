use tauri::State;
use xrf_engine_target::XrayEngineResolution;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, detect_roots_engine};
use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;

/// Detect which engine roots target, as an open left to detection would read them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "detect_engine"))]
#[tauri::command(rename = "detect_engine")]
pub async fn assets_detect_engine(
  roots: XrayRoots,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<XrayEngineResolution> {
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let resolution: XrayEngineResolution = execution
    .run_blocking("Detecting the engine", move || {
      let resolution: XrayEngineResolution = detect_roots_engine(&assets, &roots);

      log::info!("Detected roots {} as {}", roots.describe(), resolution.describe());

      resolution
    })
    .await?;

  Ok(resolution)
}
