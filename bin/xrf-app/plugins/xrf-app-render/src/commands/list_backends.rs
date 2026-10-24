use xrf_app_core::TauriResult;
use xrf_renderer::RenderBackendAvailability;

/// Say which graphics backends the renderer can draw with on this machine, and on which GPU, for the settings to offer
/// only those; each is probed off the render thread.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_backends"))]
#[tauri::command(rename = "list_backends")]
pub async fn render_list_backends() -> TauriResult<Vec<RenderBackendAvailability>> {
  tauri::async_runtime::spawn_blocking(xrf_renderer::probe_render_backends)
    .await
    .map_err(|error| format!("Probing the graphics backends failed: {error}"))
}
