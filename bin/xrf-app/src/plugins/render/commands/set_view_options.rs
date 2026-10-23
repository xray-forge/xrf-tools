use tauri::State;
use xrf_renderer::{RenderViewOptions, RenderViewportId};

use crate::plugins::render::renderer::Renderer;

/// Set what one viewport draws its scene with.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_view_options"))]
#[tauri::command(rename = "set_view_options")]
pub fn render_set_view_options(renderer: State<'_, Renderer>, viewport: RenderViewportId, options: RenderViewOptions) {
  renderer.set_view_options(viewport, options);
}
