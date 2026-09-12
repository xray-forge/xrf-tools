use tauri::State;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::spawn::state::SpawnFileState;

/// Close the committed file and prevent unfinished opens from restoring it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_file"))]
#[tauri::command(rename = "close_file")]
pub async fn spawn_close_file(
  session_ids: Vec<SessionId>,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let closed = state.close(&session_ids)?;

  execution.run_blocking("Closing spawn", move || drop(closed)).await
}
