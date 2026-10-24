use std::sync::Arc;

use tauri::State;
use xrf_app_core::session::{SessionScope, SessionScopes};
use xrf_app_core::types::TauriResult;

use crate::descriptor::ConfigsProjectDescriptor;
use crate::state::ConfigsState;

/// What configs project a tool holds, the newest it opened, for a page attaching to it after a reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub fn configs_get_project(
  state: State<'_, ConfigsState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<Option<Arc<ConfigsProjectDescriptor>>> {
  Ok(
    state
      .restore(&scopes.list(&scope))?
      .map(|opened| Arc::clone(&opened.descriptor)),
  )
}
