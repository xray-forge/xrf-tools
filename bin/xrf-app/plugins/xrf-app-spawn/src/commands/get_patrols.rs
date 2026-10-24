use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_spawn::SpawnPatrolsChunk;

use crate::state::{SpawnFileState, SpawnSession};

/// Read patrols from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_patrols"))]
#[tauri::command(rename = "get_patrols")]
pub async fn spawn_get_patrols(
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnPatrolsChunk> {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.files.require(session_id)?;

  execution
    .run_blocking("Reading spawn patrols", move || opened.file.patrols.clone())
    .await
}
