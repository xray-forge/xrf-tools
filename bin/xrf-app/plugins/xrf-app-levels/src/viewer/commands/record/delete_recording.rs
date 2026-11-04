use std::fs;
use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;

use crate::viewer::LevelViewerFolder;

/// Delete a recording of the recordings folder for good.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "delete_recording")]
pub async fn levels_delete_recording(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  path: PathBuf,
) -> TauriResult {
  LevelViewerFolder::require_recording(&directories, &path)?;

  execution
    .run_blocking("Deleting a recording", move || {
      fs::remove_file(&path).map_err(|error| format!("Failed to delete '{}': {error}", path.display()))
    })
    .await?
}
