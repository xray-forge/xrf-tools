use tauri::State;
use xrf_renderer::{RenderOverlay, RenderViewportId};

use crate::renderer::Renderer;

/// Set the helpers drawn over one viewport's frame.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_overlays"))]
#[tauri::command(rename = "set_overlays")]
pub fn render_set_overlays(renderer: State<'_, Renderer>, viewport: RenderViewportId, overlays: Vec<RenderOverlay>) {
  renderer.set_overlays(viewport, overlays);
}
