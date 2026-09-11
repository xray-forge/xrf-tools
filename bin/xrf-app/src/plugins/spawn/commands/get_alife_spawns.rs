use std::sync::Arc;

use tauri::State;
use xrf_db::SpawnALifeSpawnsChunk;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionId;
use crate::plugins::spawn::state::{SpawnFileState, SpawnSession};

/// Read alife_spawn from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_alife_spawns"))]
#[tauri::command(rename = "get_alife_spawns")]
pub async fn spawn_get_alife_spawns(
  session_id: SpawnSessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnALifeSpawnsChunk> {
  let opened: Arc<SpawnSession> = state.require(session_id)?;

  execution
    .run_blocking("Reading spawn alife_spawn", move || opened.file.alife_spawn.clone())
    .await
}
