use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_utils::error_to_string;

use crate::{SpawnFiles, SpawnSession};

/// Write the spawn `session_id` holds into the directory `path` as unpacked chunks, on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_unpacked_directory"))]
#[tauri::command(rename = "save_unpacked_directory")]
pub async fn spawn_save_unpacked_directory(
  path: PathBuf,
  session_id: SessionId,
  files: State<'_, SpawnFiles>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let opened: Arc<SessionSnapshot<SpawnSession>> = files.require(session_id)?;

  execution
    .run_blocking("Writing an unpacked spawn", move || {
      opened
        .file
        .export_to_path::<XRayByteOrder, _>(&path)
        .map_err(error_to_string)
    })
    .await?
}
