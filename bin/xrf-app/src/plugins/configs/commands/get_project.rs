use std::sync::Arc;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsProjectDescriptor;
use crate::plugins::configs::state::ConfigsState;

/// What configs project is open, for a frontend restoring itself after a reload.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub fn configs_get_project(state: State<'_, ConfigsState>) -> TauriResult<Option<Arc<ConfigsProjectDescriptor>>> {
  Ok(state.get()?.map(|opened| Arc::clone(&opened.descriptor)))
}
