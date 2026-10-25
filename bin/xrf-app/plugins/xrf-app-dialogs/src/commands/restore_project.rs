use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;
use xrf_dialog::DialogProjectDescriptor;

use crate::{DialogProjects, OpenedDialogProject};

/// The dialogs project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
/// reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_project")]
pub async fn dialogs_restore_project(
  projects: State<'_, DialogProjects>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<Arc<DialogProjectDescriptor>>> {
  Ok(
    projects
      .restore_as(&SessionScope::of(&window, tool), OpenedDialogProject::describe)
      .into(),
  )
}
