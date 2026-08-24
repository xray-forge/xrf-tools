use xrf_build_info::{BuildInfo, build_info};

/// Report which build of the application is running.
///
/// Expanded here rather than in the shared crate because `env!` resolves in the crate being compiled,
/// so this is the only place that can see what the application's own build script recorded. Cannot
/// fail, so it answers with the description directly instead of a result the caller has to unwrap.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_build_info"))]
#[tauri::command(rename = "get_build_info")]
pub fn system_get_build_info() -> BuildInfo {
  build_info!()
}
