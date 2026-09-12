use std::sync::Arc;

use tauri::State;
use xrf_db::SpawnArtefactSpawnsChunk;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::spawn::state::{SpawnFileState, SpawnSession};

/// Read artefact_spawn from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_artefact_spawns"))]
#[tauri::command(rename = "get_artefact_spawns")]
pub async fn spawn_get_artefact_spawns(
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnArtefactSpawnsChunk> {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.session.require(session_id)?;

  execution
    .run_blocking("Reading spawn artefact_spawn", move || {
      opened.file.artefact_spawn.clone()
    })
    .await
}
