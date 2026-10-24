use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::{SpawnChunk, SpawnChunkKind, SpawnFiles};

/// One large chunk of the spawn `session_id` holds, copied on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_chunk"))]
#[tauri::command(rename = "read_chunk")]
pub async fn spawn_read_chunk(
  session_id: SessionId,
  chunk: SpawnChunkKind,
  files: State<'_, SpawnFiles>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SpawnChunk>> {
  files
    .read_on_pool(&execution, session_id, "Reading a spawn chunk", move |opened| {
      Ok(opened.read_chunk(chunk))
    })
    .await
}
