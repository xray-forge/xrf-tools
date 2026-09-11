use std::sync::Arc;

use tauri::State;
use xrf_db::SpawnGraphsChunk;

use crate::core::execution::ExecutionState;
use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::spawn::state::{SpawnFileState, SpawnSession};

/// Read graphs from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_graphs"))]
#[tauri::command(rename = "get_graphs")]
pub async fn spawn_get_graphs(
  session_id: DocumentSessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnGraphsChunk> {
  let opened: Arc<DocumentSnapshot<SpawnSession>> = state.require(session_id)?;

  execution
    .run_blocking("Reading spawn graphs", move || opened.file.graphs.clone())
    .await
}
