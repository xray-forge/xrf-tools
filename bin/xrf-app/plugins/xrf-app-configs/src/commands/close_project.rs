use tauri::State;
use xrf_app_core::session::SessionId;

use crate::ConfigsProjects;

/// Close the configs projects `session_ids` name, committed or opening, and whatever each had resolved.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_project")]
pub fn configs_close_project(projects: State<'_, ConfigsProjects>, session_ids: Vec<SessionId>) {
  projects.close(&session_ids);
}
