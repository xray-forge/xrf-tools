use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_spawn::SpawnALifeSpawnsChunk;

use crate::state::{SpawnFileState, SpawnSession};

/// Read alife_spawn from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_alife_spawns"))]
#[tauri::command(rename = "get_alife_spawns")]
pub async fn spawn_get_alife_spawns(
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnALifeSpawnsChunk> {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.files.require(session_id)?;

  execution
    .run_blocking("Reading spawn alife_spawn", move || opened.file.alife_spawn.clone())
    .await
}
