use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::ConfigsState;

/// Closes the configs project, releasing its mounts and whatever it had resolved.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub fn configs_close_project(session_ids: Vec<SessionId>, state: State<'_, ConfigsState>) -> TauriResult<()> {
  log::info!("Closing ltx configs project");

  state.close(&session_ids)
}
