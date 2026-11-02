use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;
use xrf_export::ExportsProject;

use crate::ExportsProjects;

/// The exports project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
/// reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_project")]
pub async fn exports_restore_project(
  projects: State<'_, ExportsProjects>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<ExportsProject>> {
  Ok(projects.restore(&SessionScope::of(&window, tool)).into())
}
