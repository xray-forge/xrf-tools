use std::path::PathBuf;

use tauri::State;
use xrf_app_core::app_directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::types::TauriResult;

use crate::viewer::LevelViewerFolder;
use crate::viewer::cube_export::export_cube;
use crate::viewer::cube_export_request::CubeExportRequest;

/// Write a cubemap of the screenshots folder again as a cube DDS, six TGAs or six PNGs; answers the files written.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "export_cube"))]
#[tauri::command(rename = "export_cube")]
pub async fn levels_export_cube(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  request: CubeExportRequest,
) -> TauriResult<Vec<PathBuf>> {
  LevelViewerFolder::require_screenshot(&directories, &request.path)?;

  log::info!(
    "Exporting cubemap '{}' as '{}'",
    request.path.display(),
    request.target.display()
  );

  execution
    .run_blocking("Exporting a cubemap", move || export_cube(&request))
    .await?
}
