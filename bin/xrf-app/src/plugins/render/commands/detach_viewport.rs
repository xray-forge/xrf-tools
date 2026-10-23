use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::plugins::render::render_viewports::RenderViewports;

/// Stop drawing a viewport; the GPU goes a few seconds after the last one.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "detach_viewport"))]
#[tauri::command(rename = "detach_viewport")]
pub fn render_detach_viewport(viewports: State<'_, RenderViewports>, viewport: RenderViewportId) {
  viewports.detach(viewport);
}
