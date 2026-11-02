use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;

use crate::paths::PathDescription;

/// Describe what a path currently holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_path")]
pub async fn system_describe_path(execution: State<'_, ExecutionState>, path: PathBuf) -> TauriResult<PathDescription> {
  // A directory's entries are counted, which grows with it.
  execution
    .run_blocking("Describing a path", move || {
      PathDescription::read(&path).map_err(|error| format!("Could not read {}: {error}", path.display()))
    })
    .await?
}
