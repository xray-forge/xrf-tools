use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::levels::viewer::map_export::export_map;
use crate::plugins::levels::viewer::map_export_request::MapExportRequest;
use crate::plugins::system::paths::SystemPathsState;

/// Write a level map of the screenshots folder again as a PNG, a TGA or a DDS, or its annotations alone as an SVG;
/// answers its pixels across and down.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "export_map"))]
#[tauri::command(rename = "export_map")]
pub async fn levels_export_map(
  state: State<'_, SystemPathsState>,
  execution: State<'_, ExecutionState>,
  request: MapExportRequest,
) -> TauriResult<(u32, u32)> {
  LevelViewerFolder::require_screenshot(state.local_data.clone(), &request.path)?;

  log::info!(
    "Exporting level map '{}' as '{}'",
    request.path.display(),
    request.target.display()
  );

  execution
    .run_blocking("Exporting a level map", move || export_map(&request))
    .await?
}
