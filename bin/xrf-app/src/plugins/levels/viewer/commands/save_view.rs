use std::path::PathBuf;

use tauri::State;
use xrf_utils::write_file_staged;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;

/// Write a level view, or a screenshot's report, as its JSON text where the page names.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_view"))]
#[tauri::command(rename = "save_view")]
pub async fn levels_save_view(execution: State<'_, ExecutionState>, path: PathBuf, text: String) -> TauriResult {
  execution
    .run_blocking("Saving a level view", move || {
      write_file_staged(&path, text.as_bytes())
        .map_err(|error| format!("Failed to write the view '{}': {error}", path.display()))
    })
    .await?
}
