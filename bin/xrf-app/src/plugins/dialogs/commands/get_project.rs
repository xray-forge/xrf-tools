use tauri::State;
use xrf_dialog::{DialogProject, DialogProjectDescriptor};

use crate::core::session::SessionRestore;
use crate::core::types::TauriResult;
use crate::plugins::dialogs::state::DialogProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn dialogs_get_project(
  state: State<'_, DialogProjectState>,
) -> TauriResult<SessionRestore<DialogProjectDescriptor>> {
  Ok(SessionRestore::from(
    state.get()?.map(|opened| opened.map(DialogProject::describe)),
  ))
}
