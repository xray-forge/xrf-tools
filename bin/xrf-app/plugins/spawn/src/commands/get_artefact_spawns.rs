use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_spawn::SpawnArtefactSpawnsChunk;

use crate::state::{SpawnFileState, SpawnSession};

/// Read artefact_spawn from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_artefact_spawns"))]
#[tauri::command(rename = "get_artefact_spawns")]
pub async fn spawn_get_artefact_spawns(
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnArtefactSpawnsChunk> {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.files.require(session_id)?;

  execution
    .run_blocking("Reading spawn artefact_spawn", move || {
      opened.file.artefact_spawn.clone()
    })
    .await
}
