use std::sync::Arc;

use tauri::State;
use xrf_db::SpawnFile;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionId;
use crate::plugins::spawn::state::{SpawnFileState, SpawnSession};

/// Read the whole file from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_file"))]
#[tauri::command(rename = "get_file")]
pub async fn spawn_get_file(
  session_id: SpawnSessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnFile> {
  let opened: Arc<SpawnSession> = state.require(session_id)?;

  execution
    .run_blocking("Reading spawn file", move || opened.file.clone())
    .await
}
