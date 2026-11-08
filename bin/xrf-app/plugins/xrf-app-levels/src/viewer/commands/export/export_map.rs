use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;

use crate::viewer::map::{MapExportOutcome, MapExportRequest, MapExporter};
use crate::viewer::{LevelViewerFolder, LevelViewerReport};

/// Write a level map of the screenshots folder again as a PNG, a TGA or a DDS, or its annotations alone as an SVG;
/// answers its size.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "export_map")]
pub(crate) async fn levels_export_map(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  request: MapExportRequest,
) -> TauriResult<MapExportOutcome> {
  LevelViewerFolder::require_screenshot(&directories, &request.path)?;
  LevelViewerReport::exporting_map(&request.path, &request.target);

  execution
    .run_blocking("Exporting a level map", move || MapExporter::export(&request))
    .await?
}
