use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;

/// Where tools write when no output directory has been configured.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_default_output_root"))]
#[tauri::command(rename = "get_default_output_root")]
pub async fn system_get_default_output_root(directories: State<'_, AppDirectories>) -> TauriResult<String> {
  directories
    .get_output_root()
    .ok_or_else(|| String::from("No available output root has a Unicode path"))
}
