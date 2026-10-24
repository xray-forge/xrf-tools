use tauri::State;
use xrf_app_core::session::SessionId;

use crate::TranslationProjects;

/// Close the translations projects `session_ids` name, committed or opening; a closed one answers no read and takes no
/// save.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub fn translations_close_project(projects: State<'_, TranslationProjects>, session_ids: Vec<SessionId>) {
  projects.close(&session_ids);
}
