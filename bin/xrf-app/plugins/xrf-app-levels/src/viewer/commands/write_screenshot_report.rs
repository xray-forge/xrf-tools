use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_utils::StagedFile;

use crate::viewer::LevelViewerFolder;

/// Write a screenshot's report beside it in the screenshots folder, as its JSON text; answers where it was written.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "write_screenshot_report")]
pub async fn levels_write_screenshot_report(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  path: PathBuf,
  text: String,
) -> TauriResult<String> {
  LevelViewerFolder::require_screenshot(&directories, &path)?;

  let report: PathBuf = LevelViewerFolder::get_report_path(&path);

  execution
    .run_blocking("Writing a screenshot's report", move || {
      StagedFile::write(&report, text.as_bytes())
        .map(|()| report.display().to_string())
        .map_err(|error| format!("Failed to write the report '{}': {error}", report.display()))
    })
    .await?
}
