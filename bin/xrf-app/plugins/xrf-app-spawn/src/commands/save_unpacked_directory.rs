use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_utils::{OutputFiles, error_to_string};

use crate::{SpawnFiles, SpawnSession};

/// Write the spawn `session_id` holds into the directory `path` as unpacked chunks; refused where `path` exists, in
/// any letter case, unless `is_forced`.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_unpacked_directory")]
pub(crate) async fn spawn_save_unpacked_directory(
  path: PathBuf,
  session_id: SessionId,
  is_forced: bool,
  files: State<'_, SpawnFiles>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let opened: Arc<SessionSnapshot<SpawnSession>> = files.require(session_id)?;

  execution
    .run_blocking("Writing an unpacked spawn", move || {
      OutputFiles::of_path(&path)
        .assert_free(is_forced)
        .and_then(|()| opened.file.export_to_path::<XRayByteOrder, _>(&path))
        .map_err(error_to_string)
    })
    .await?
}
