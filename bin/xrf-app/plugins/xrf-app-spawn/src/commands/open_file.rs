use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::{SpawnFiles, SpawnSession, SpawnSessionDescriptor};

/// Open the packed spawn at `path`, read on the pool, answering what describes it without its large chunks.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_file")]
pub async fn spawn_open_file(
  session_id: SessionId,
  path: PathBuf,
  files: State<'_, SpawnFiles>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SpawnSessionDescriptor>> {
  log::info!("Opening spawn '{}'", path.display());

  let opened = files
    .open_on_pool(&execution, session_id, "Opening a spawn", move || {
      SpawnSession::read(path)
    })
    .await?;

  Ok(opened.map(SpawnSession::describe))
}
