use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_utils::{OutputFiles, error_to_string};

use crate::{SpawnFiles, SpawnSession};

/// Write the spawn `session_id` holds to `path` as a packed file; refused where `path` exists, in any letter case,
/// unless `is_forced`.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_file")]
pub(crate) async fn spawn_save_file(
  path: PathBuf,
  session_id: SessionId,
  is_forced: bool,
  files: State<'_, SpawnFiles>,
  execution: State<'_, ExecutionState>,
) -> TauriResult {
  let opened: Arc<SessionSnapshot<SpawnSession>> = files.require(session_id)?;

  execution
    .run_blocking("Writing a spawn", move || {
      OutputFiles::of_path(&path)
        .assert_free(is_forced)
        .and_then(|()| opened.file.write_to_path(&path))
        .map_err(error_to_string)
    })
    .await?
}
