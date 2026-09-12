use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_db::XRayByteOrder;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::spawn::state::{SpawnFileState, SpawnSession};

/// Write the requested session using the existing spawn format writer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_file"))]
#[tauri::command(rename = "save_file")]
pub async fn spawn_save_file(
  path: PathBuf,
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.session.require(session_id)?;

  execution
    .run_blocking("Writing spawn", move || {
      opened
        .file
        .write_to_path::<XRayByteOrder, _>(&path)
        .map_err(error_to_string)
    })
    .await?
}
