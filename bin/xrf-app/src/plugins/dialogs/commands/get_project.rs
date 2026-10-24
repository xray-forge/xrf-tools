use tauri::State;
use xrf_dialog::{DialogProject, DialogProjectDescriptor};

use crate::core::session::{SessionRestore, SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::dialogs::state::DialogProjectState;

/// What dialogs project a tool holds, the newest it opened, for a page attaching to it after a reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn dialogs_get_project(
  state: State<'_, DialogProjectState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<DialogProjectDescriptor>> {
  Ok(SessionRestore::from(
    state
      .restore(&scopes.list(&scope))?
      .map(|opened| opened.map(DialogProject::describe)),
  ))
}
