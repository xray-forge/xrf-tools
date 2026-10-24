use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;

/// Where the application keeps its own files, as its logs are kept: its first data directory with a Unicode path.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_app_data_root"))]
#[tauri::command(rename = "get_app_data_root")]
pub async fn system_get_app_data_root(directories: State<'_, AppDirectories>) -> TauriResult<String> {
  directories
    .get_app_data_root()
    .ok_or_else(|| String::from("No application data directory has a Unicode path"))
}
