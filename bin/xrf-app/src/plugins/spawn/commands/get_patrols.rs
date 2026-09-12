use std::sync::Arc;

use tauri::State;
use xrf_db::SpawnPatrolsChunk;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::spawn::state::{SpawnFileState, SpawnSession};

/// Read patrols from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_patrols"))]
#[tauri::command(rename = "get_patrols")]
pub async fn spawn_get_patrols(
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnPatrolsChunk> {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.require(session_id)?;

  execution
    .run_blocking("Reading spawn patrols", move || opened.file.patrols.clone())
    .await
}
