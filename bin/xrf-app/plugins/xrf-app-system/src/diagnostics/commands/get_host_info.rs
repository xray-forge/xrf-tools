use xrf_app_core::TauriResult;

use crate::diagnostics::HostInfo;

/// Report what the application is running on and with.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_host_info"))]
#[tauri::command(rename = "get_host_info")]
pub async fn system_get_host_info() -> TauriResult<HostInfo> {
  // The system and the webview are asked, which no async worker should wait through.
  tauri::async_runtime::spawn_blocking(HostInfo::read)
    .await
    .map_err(|error| format!("The host could not be read: {error}"))
}
