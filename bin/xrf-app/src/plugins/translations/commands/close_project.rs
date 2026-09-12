use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::translations::state::TranslationProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub async fn translations_close_project(
  session_ids: Vec<SessionId>,
  state: State<'_, TranslationProjectState>,
) -> TauriResult {
  log::info!("Closing translations project");

  state.close_project(&session_ids)
}
