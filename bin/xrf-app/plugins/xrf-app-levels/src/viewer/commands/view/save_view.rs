use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_utils::StagedFile;

use crate::viewer::LevelViewerFolder;

/// Write a level view as its JSON text to the JSON file the page names.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_view")]
pub async fn levels_save_view(execution: State<'_, ExecutionState>, path: PathBuf, text: String) -> TauriResult {
  LevelViewerFolder::require_view_file(&path)?;

  execution
    .run_blocking("Saving a level view", move || {
      StagedFile::write(&path, text.as_bytes())
        .map_err(|error| format!("Failed to write the view '{}': {error}", path.display()))
    })
    .await?
}
