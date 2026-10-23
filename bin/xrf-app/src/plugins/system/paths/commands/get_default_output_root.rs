use tauri::State;

use crate::core::app_directories::AppDirectories;
use crate::core::directories::get_first_unicode_path;
use crate::core::types::TauriResult;

/// Where tools write when no output directory has been configured.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_default_output_root"))]
#[tauri::command(rename = "get_default_output_root")]
pub async fn system_get_default_output_root(directories: State<'_, AppDirectories>) -> TauriResult<String> {
  get_first_unicode_path(directories.get_output_directories())
    .ok_or_else(|| String::from("No available output root has a Unicode path"))
}
