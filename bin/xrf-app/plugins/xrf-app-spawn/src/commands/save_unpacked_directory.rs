use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_spawn::XRayByteOrder;
use xrf_utils::error_to_string;

use crate::state::{SpawnFileState, SpawnSession};

/// Write the requested session using the existing spawn format writer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_unpacked_directory"))]
#[tauri::command(rename = "save_unpacked_directory")]
pub async fn spawn_save_unpacked_directory(
  path: PathBuf,
  session_id: SessionId,
  state: State<'_, SpawnFileState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let opened: Arc<SessionSnapshot<SpawnSession>> = state.files.require(session_id)?;

  execution
    .run_blocking("Writing spawn", move || {
      opened
        .file
        .export_to_path::<XRayByteOrder, _>(&path)
        .map_err(error_to_string)
    })
    .await?
}
