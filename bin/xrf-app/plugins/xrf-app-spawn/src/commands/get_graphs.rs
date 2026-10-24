use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_spawn::SpawnGraphsChunk;

use crate::state::{SpawnFileState, SpawnSession};

/// Read graphs from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_graphs"))]
#[tauri::command(rename = "get_graphs")]
pub async fn spawn_get_graphs(
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnGraphsChunk> {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.files.require(session_id)?;

  execution
    .run_blocking("Reading spawn graphs", move || opened.file.graphs.clone())
    .await
}
