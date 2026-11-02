use tauri::State;
use xrf_build_info::BuildInfo;

/// Report which build of the application is running, as the application recorded it.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "get_build_info")]
pub fn system_get_build_info(build: State<'_, BuildInfo>) -> BuildInfo {
  build.inner().clone()
}
