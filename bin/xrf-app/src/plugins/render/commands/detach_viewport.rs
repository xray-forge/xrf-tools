use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::plugins::render::state::RenderState;

/// Stop drawing a viewport; the GPU goes a few seconds after the last one.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "detach_viewport"))]
#[tauri::command(rename = "detach_viewport")]
pub fn render_detach_viewport(state: State<'_, RenderState>, viewport: RenderViewportId) {
  log::info!("Detached native viewport {}", viewport.0);

  state.renderer.detach_viewport(viewport);
  state.forget(viewport);
}
