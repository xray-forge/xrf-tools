use crate::plugins::system::diagnostics::HostInfo;

/// Report what the application is running on and with.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_host_info"))]
#[tauri::command(rename = "get_host_info")]
pub async fn system_get_host_info() -> HostInfo {
  HostInfo::read()
}
