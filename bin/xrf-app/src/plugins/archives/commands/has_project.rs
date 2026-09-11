use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "has_project"))]
#[tauri::command(rename = "has_project")]
pub fn archives_has_project(state: State<'_, ArchiveProjectState>) -> TauriResult<bool> {
  log::debug!("Checking archives project presence");

  Ok(state.get()?.is_some())
}
