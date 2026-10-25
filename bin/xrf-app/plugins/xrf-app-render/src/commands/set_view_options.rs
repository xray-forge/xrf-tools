use tauri::State;
use xrf_renderer::{RenderViewOptions, RenderViewportId};

use crate::Renderer;

/// Set what one viewport draws its scene with.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "set_view_options")]
pub fn render_set_view_options(renderer: State<'_, Renderer>, viewport: RenderViewportId, options: RenderViewOptions) {
  renderer.set_view_options(viewport, options);
}
