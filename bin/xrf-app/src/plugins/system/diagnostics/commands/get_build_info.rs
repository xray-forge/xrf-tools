use xrf_build_info::{BuildInfo, build_info};

/// Report which build of the application is running.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_build_info"))]
#[tauri::command(rename = "get_build_info")]
pub fn system_get_build_info() -> BuildInfo {
  build_info!()
}
