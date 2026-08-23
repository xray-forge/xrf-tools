use std::sync::MutexGuard;

use tauri::State;
use xrf_dialog::{DialogProject, DialogProjectDescriptor};

use crate::core::types::TauriResult;
use crate::plugins::dialogs::state::DialogProjectState;

/// The open project, described again rather than cached.
///
/// Provisioning asks the backend what is open, so a reload restores the session; the descriptor is
/// derived from the project on demand because the project is what state owns.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn dialogs_get_project(state: State<'_, DialogProjectState>) -> TauriResult<Option<DialogProjectDescriptor>> {
  let lock: MutexGuard<Option<DialogProject>> = state.project.lock().unwrap();

  Ok(lock.as_ref().map(DialogProject::describe))
}
