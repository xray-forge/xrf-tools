use std::fs;
use std::path::PathBuf;

use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;

/// Read a level view's JSON file as text, for the page to check and import.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_view"))]
#[tauri::command(rename = "read_view")]
pub async fn levels_read_view(execution: State<'_, ExecutionState>, path: PathBuf) -> TauriResult<String> {
  LevelViewerFolder::require_view_file(&path)?;

  execution
    .run_blocking("Reading a level view", move || {
      fs::read_to_string(&path).map_err(|error| format!("Failed to read the view '{}': {error}", path.display()))
    })
    .await?
}
