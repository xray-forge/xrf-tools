use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::dialogs::state::DialogProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub async fn dialogs_close_project(state: State<'_, DialogProjectState>) -> TauriResult {
  log::info!("Closing dialogs project");

  *state.project.lock().unwrap() = None;

  Ok(())
}
