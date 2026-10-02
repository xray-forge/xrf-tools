use tauri::State;
use xrf_renderer::{RenderViewportId, RenderViewportLayout};

use crate::plugins::render::state::RenderState;

/// Place a viewport where its element now is, and say what the page shows around it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_viewport_layout"))]
#[tauri::command(rename = "set_viewport_layout")]
pub fn render_set_viewport_layout(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
  layout: RenderViewportLayout,
) {
  state.renderer.set_layout(viewport, layout);
}
