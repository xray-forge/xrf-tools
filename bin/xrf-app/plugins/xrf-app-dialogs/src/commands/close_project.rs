use tauri::State;
use xrf_app_core::session::SessionId;

use crate::DialogProjects;

/// Close the dialogs projects `session_ids` name, committed or opening; a closed one answers no read.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_project")]
pub fn dialogs_close_project(projects: State<'_, DialogProjects>, session_ids: Vec<SessionId>) {
  projects.close(&session_ids);
}
