use std::path::PathBuf;

use tauri::State;
use xrf_utils::write_file_staged;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;

/// Write a level view as its JSON text to the JSON file the page names.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_view"))]
#[tauri::command(rename = "save_view")]
pub async fn levels_save_view(execution: State<'_, ExecutionState>, path: PathBuf, text: String) -> TauriResult {
  LevelViewerFolder::require_view_file(&path)?;

  execution
    .run_blocking("Saving a level view", move || {
      write_file_staged(&path, text.as_bytes())
        .map_err(|error| format!("Failed to write the view '{}': {error}", path.display()))
    })
    .await?
}
