use tauri::State;

use crate::core::app_directories::AppDirectories;
use crate::core::directories::get_first_unicode_path;
use crate::core::types::TauriResult;

/// Where the application keeps its own files, as its logs are kept: its first data directory with a Unicode path.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_app_data_root"))]
#[tauri::command(rename = "get_app_data_root")]
pub async fn system_get_app_data_root(directories: State<'_, AppDirectories>) -> TauriResult<String> {
  get_first_unicode_path(directories.get_app_data_directories())
    .ok_or_else(|| String::from("No application data directory has a Unicode path"))
}
