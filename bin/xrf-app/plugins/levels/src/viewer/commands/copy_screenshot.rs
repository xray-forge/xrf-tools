use std::fs;
use std::path::PathBuf;

use tauri::State;
use xrf_app_core::app_directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::types::TauriResult;

use crate::viewer::LevelViewerFolder;

/// Copy a screenshot of the screenshots folder to where the page names, as Save as does.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "copy_screenshot"))]
#[tauri::command(rename = "copy_screenshot")]
pub async fn levels_copy_screenshot(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  path: PathBuf,
  target: PathBuf,
) -> TauriResult {
  LevelViewerFolder::require_screenshot(&directories, &path)?;

  execution
    .run_blocking("Copying a screenshot", move || {
      fs::copy(&path, &target)
        .map(|_| ())
        .map_err(|error| format!("Failed to copy '{}' to '{}': {error}", path.display(), target.display()))
    })
    .await?
}
