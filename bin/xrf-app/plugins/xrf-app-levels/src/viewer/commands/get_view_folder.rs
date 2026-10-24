use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;

use crate::viewer::LevelViewerFolder;

/// The folder the level viewer's views are exported to and imported from by default, made where it is not yet.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_view_folder"))]
#[tauri::command(rename = "get_view_folder")]
pub async fn levels_get_view_folder(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<String> {
  let folder: PathBuf = LevelViewerFolder::Views.get_path(&directories)?;

  execution
    .run_blocking("Making the views folder", move || {
      LevelViewerFolder::make(folder).map(|folder: PathBuf| folder.display().to_string())
    })
    .await?
}
