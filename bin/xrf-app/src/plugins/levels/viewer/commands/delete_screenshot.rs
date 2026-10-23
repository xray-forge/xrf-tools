use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use tauri::State;

use crate::core::app_directories::AppDirectories;
use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;

/// Delete a screenshot of the screenshots folder for good, with what stands beside it: a screenshot's report, a level
/// map's annotations.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "delete_screenshot"))]
#[tauri::command(rename = "delete_screenshot")]
pub async fn levels_delete_screenshot(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  path: PathBuf,
) -> TauriResult {
  LevelViewerFolder::require_screenshot(&directories, &path)?;

  execution
    .run_blocking("Deleting a screenshot", move || {
      fs::remove_file(&path).map_err(|error| format!("Failed to delete '{}': {error}", path.display()))?;

      for beside in [
        LevelViewerFolder::get_report_path(&path),
        LevelViewerFolder::get_annotations_path(&path),
      ] {
        match fs::remove_file(&beside) {
          Err(error) if error.kind() != ErrorKind::NotFound => {
            return Err(format!("Failed to delete '{}': {error}", beside.display()));
          }
          _ => {}
        }
      }

      Ok(())
    })
    .await?
}
