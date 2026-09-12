use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::configs::state::ConfigsState;

/// Closes the configs project, releasing its mounts and whatever it had resolved.
///
/// Reissues the session identity, so a read already in flight cannot commit against the project that follows.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub fn configs_close_project(session_ids: Vec<SessionId>, state: State<'_, ConfigsState>) -> TauriResult<()> {
  log::info!("Closing ltx configs project");

  state.close(&session_ids)
}
