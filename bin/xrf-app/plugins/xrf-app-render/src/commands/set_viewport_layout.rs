use tauri::State;
use xrf_renderer::{RenderViewportId, RenderViewportLayout};

use crate::Renderer;

/// Place a viewport where its element now is, and say what the page shows around it.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "set_viewport_layout")]
pub fn render_set_viewport_layout(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
  layout: RenderViewportLayout,
) {
  renderer.set_layout(viewport, layout);
}
