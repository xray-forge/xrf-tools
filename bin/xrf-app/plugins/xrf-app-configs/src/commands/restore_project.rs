use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::{ConfigsProject, ConfigsProjectDescriptor, ConfigsProjects};

/// The configs project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
/// reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_project")]
pub async fn configs_restore_project(
  projects: State<'_, ConfigsProjects>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<Arc<ConfigsProjectDescriptor>>> {
  Ok(
    projects
      .restore_as(&SessionScope::of(&window, tool), ConfigsProject::describe)
      .into(),
  )
}
