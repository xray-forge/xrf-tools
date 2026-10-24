use tauri::State;
use xrf_app_core::session::SessionId;

use crate::ExportsProjects;

/// Close the exports projects `session_ids` name, committed or opening; a closed one answers no read.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_project"))]
#[tauri::command(rename = "close_project")]
pub fn exports_close_project(projects: State<'_, ExportsProjects>, session_ids: Vec<SessionId>) {
  projects.close(&session_ids);
}
