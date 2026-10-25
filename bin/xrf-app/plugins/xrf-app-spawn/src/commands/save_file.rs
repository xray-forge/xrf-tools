use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_utils::error_to_string;

use crate::{SpawnFiles, SpawnSession};

/// Write the spawn `session_id` holds to `path` as a packed file, on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_file")]
pub async fn spawn_save_file(
  path: PathBuf,
  session_id: SessionId,
  files: State<'_, SpawnFiles>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let opened: Arc<SessionSnapshot<SpawnSession>> = files.require(session_id)?;

  execution
    .run_blocking("Writing a spawn", move || {
      opened
        .file
        .write_to_path::<XRayByteOrder, _>(&path)
        .map_err(error_to_string)
    })
    .await?
}
