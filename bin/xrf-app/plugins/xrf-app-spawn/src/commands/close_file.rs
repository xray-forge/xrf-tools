use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::SpawnFileState;

/// Close the committed file and prevent unfinished opens from restoring it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_file"))]
#[tauri::command(rename = "close_file")]
pub async fn spawn_close_file(
  session_ids: Vec<SessionId>,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let closed = state.files.detach(&session_ids)?;

  execution.run_blocking("Closing spawn", move || drop(closed)).await
}
