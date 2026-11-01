use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;

use crate::viewer::cube::{CubeExportRequest, CubeExporter};
use crate::viewer::{LevelViewerFolder, LevelViewerReport};

/// Write a cubemap of the screenshots folder again as a cube DDS, six TGAs or six PNGs; answers the files written.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "export_cube")]
pub async fn levels_export_cube(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  request: CubeExportRequest,
) -> TauriResult<Vec<PathBuf>> {
  LevelViewerFolder::require_screenshot(&directories, &request.path)?;
  LevelViewerReport::exporting_cube(&request.path, &request.target);

  execution
    .run_blocking("Exporting a cubemap", move || CubeExporter::export(&request))
    .await?
}
