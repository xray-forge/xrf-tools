use tauri::State;
use xrf_export::ExportsProject;

use crate::core::session::SessionRestore;
use crate::core::types::TauriResult;
use crate::plugins::exports::state::ExportsProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn exports_get_project(state: State<'_, ExportsProjectState>) -> TauriResult<SessionRestore<ExportsProject>> {
  Ok(SessionRestore::from(state.get()?))
}
