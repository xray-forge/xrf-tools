use tauri::State;
use xrf_renderer::{RenderOverlay, RenderViewportId};

use crate::plugins::render::state::RenderState;

/// Set the helpers drawn over one viewport's frame.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_overlays"))]
#[tauri::command(rename = "set_overlays")]
pub fn render_set_overlays(state: State<'_, RenderState>, viewport: RenderViewportId, overlays: Vec<RenderOverlay>) {
  state.renderer.set_overlays(viewport, overlays);
}
