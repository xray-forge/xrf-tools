use std::fs;
use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_utils::StagedFile;

use crate::viewer::LevelViewerFolder;

/// Copy a screenshot of the screenshots folder to where the page names, as Save as does; read whole first, so a failed
/// copy leaves no partial file and keeps any file it would have replaced.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
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
      let bytes: Vec<u8> = fs::read(&path).map_err(|error| format!("Failed to read '{}': {error}", path.display()))?;

      StagedFile::write(&target, &bytes)
        .map_err(|error| format!("Failed to copy '{}' to '{}': {error}", path.display(), target.display()))
    })
    .await?
}
