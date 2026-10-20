use tauri::State;

use crate::core::directories::{get_first_unicode_path, get_output_directories};
use crate::core::types::TauriResult;
use crate::plugins::system::paths::SystemPathsState;

/// Where tools write when no output directory has been configured.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_default_output_root"))]
#[tauri::command(rename = "get_default_output_root")]
pub async fn system_get_default_output_root(state: State<'_, SystemPathsState>) -> TauriResult<String> {
  get_first_unicode_path(get_output_directories(state.local_data.clone()))
    .ok_or_else(|| String::from("No available output root has a Unicode path"))
}
