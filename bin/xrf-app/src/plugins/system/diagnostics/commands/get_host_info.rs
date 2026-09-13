use crate::plugins::system::diagnostics::HostInfo;

/// Report what the application is running on and with.
///
/// Cannot fail: every reading it cannot get is reported as absent, so there is no result for a caller to unwrap.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_host_info"))]
#[tauri::command(rename = "get_host_info")]
pub async fn system_get_host_info() -> HostInfo {
  HostInfo::read()
}
