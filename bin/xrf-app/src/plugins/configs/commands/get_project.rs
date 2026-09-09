use std::sync::Arc;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsProjectDescriptor;
use crate::plugins::configs::state::ConfigsState;

/// What configs project is open, for a frontend restoring itself after a reload.
///
/// Inline rather than blocking: it hands back a handle on what is already held and reads nothing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_project"))]
#[tauri::command(rename = "get_project")]
pub fn configs_get_project(state: State<'_, ConfigsState>) -> TauriResult<Option<Arc<ConfigsProjectDescriptor>>> {
  state.get_descriptor()
}
