use std::sync::Arc;

use tauri::State;
use xrf_db::SpawnHeaderChunk;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionId;
use crate::plugins::spawn::state::{SpawnFileState, SpawnSession};

/// Read header from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_header"))]
#[tauri::command(rename = "get_header")]
pub async fn spawn_get_header(
  session_id: SpawnSessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnHeaderChunk> {
  let opened: Arc<SpawnSession> = state.require(session_id)?;

  execution
    .run_blocking("Reading spawn header", move || opened.file.header.clone())
    .await
}
