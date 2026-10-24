use tauri::State;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::types::TauriResult;
use xrf_export::ExportsProject;

use crate::state::ExportsProjectState;

/// What exports project a tool holds, the newest it opened, for a page attaching to it after a reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub async fn exports_get_project(
  state: State<'_, ExportsProjectState>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<ExportsProject>> {
  Ok(SessionRestore::from(state.restore(&scope)?))
}
