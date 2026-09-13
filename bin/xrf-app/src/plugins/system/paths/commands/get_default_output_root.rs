use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::system::paths::SystemPathsState;
use crate::plugins::system::paths::output_root::default_output_root;

/// Where tools write when no output directory has been configured.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_default_output_root"))]
#[tauri::command(rename = "get_default_output_root")]
pub async fn system_get_default_output_root(state: State<'_, SystemPathsState>) -> TauriResult<String> {
  default_output_root(state.local_data.clone())
}
