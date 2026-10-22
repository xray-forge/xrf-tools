use std::path::PathBuf;

use tauri::State;
use xrf_utils::write_file_staged;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::system::paths::SystemPathsState;

/// Write a screenshot's report beside it in the screenshots folder, as its JSON text; answers where it was written.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "write_screenshot_report"))]
#[tauri::command(rename = "write_screenshot_report")]
pub async fn levels_write_screenshot_report(
  state: State<'_, SystemPathsState>,
  execution: State<'_, ExecutionState>,
  path: PathBuf,
  text: String,
) -> TauriResult<String> {
  LevelViewerFolder::require_screenshot(state.local_data.clone(), &path)?;

  let report: PathBuf = LevelViewerFolder::get_report_path(&path);

  execution
    .run_blocking("Writing a screenshot's report", move || {
      write_file_staged(&report, text.as_bytes())
        .map(|()| report.display().to_string())
        .map_err(|error| format!("Failed to write the report '{}': {error}", report.display()))
    })
    .await?
}
