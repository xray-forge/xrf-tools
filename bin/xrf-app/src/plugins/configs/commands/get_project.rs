use std::sync::Arc;

use tauri::State;

use crate::core::session::{SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsProjectDescriptor;
use crate::plugins::configs::state::ConfigsState;

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
