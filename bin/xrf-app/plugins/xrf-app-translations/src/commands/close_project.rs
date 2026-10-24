use tauri::State;
use xrf_app_core::session::SessionId;
use xrf_app_core::types::TauriResult;

use crate::state::TranslationProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub async fn translations_close_project(
  session_ids: Vec<SessionId>,
  state: State<'_, TranslationProjectState>,
) -> TauriResult {
  log::info!("Closing translations project");

  state.projects.close(&session_ids)
}
