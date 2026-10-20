use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::system::paths::SystemPathsState;

/// Delete a screenshot of the screenshots folder for good, with its report beside it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "delete_screenshot"))]
#[tauri::command(rename = "delete_screenshot")]
pub async fn levels_delete_screenshot(
  state: State<'_, SystemPathsState>,
  execution: State<'_, ExecutionState>,
  path: PathBuf,
) -> TauriResult {
  LevelViewerFolder::require_screenshot(state.local_data.clone(), &path)?;

  execution
    .run_blocking("Deleting a screenshot", move || {
      let report: PathBuf = path.with_extension(LevelViewerFolder::REPORT_EXTENSION);

      fs::remove_file(&path).map_err(|error| format!("Failed to delete '{}': {error}", path.display()))?;

      match fs::remove_file(&report) {
        Err(error) if error.kind() != ErrorKind::NotFound => {
          Err(format!("Failed to delete '{}': {error}", report.display()))
        }
        _ => Ok(()),
      }
    })
    .await?
}
