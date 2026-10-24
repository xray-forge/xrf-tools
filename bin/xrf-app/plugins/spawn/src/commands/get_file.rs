use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_spawn::SpawnFile;

use crate::state::{SpawnFileState, SpawnSession};

/// Read the whole file from the requested opening, refusing a replaced session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_file"))]
#[tauri::command(rename = "get_file")]
pub async fn spawn_get_file(
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SpawnFile> {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.files.require(session_id)?;

  execution
    .run_blocking("Reading spawn file", move || opened.file.clone())
    .await
}
