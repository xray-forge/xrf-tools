use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::{SpawnFiles, SpawnSession, SpawnSessionDescriptor};

/// The spawn `tool` holds in the asking window, the newest it opened, described without its large chunks.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_file")]
pub async fn spawn_restore_file(
  files: State<'_, SpawnFiles>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<SpawnSessionDescriptor>> {
  Ok(
    files
      .restore_as(&SessionScope::of(&window, tool), SpawnSession::describe)
      .into(),
  )
}
