use tauri::State;
use xrf_renderer::{RenderViewOptions, RenderViewportId};

use crate::plugins::render::state::RenderState;

/// Set what one viewport draws its scene with.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_view_options"))]
#[tauri::command(rename = "set_view_options")]
pub fn render_set_view_options(state: State<'_, RenderState>, viewport: RenderViewportId, options: RenderViewOptions) {
  state.renderer.set_view_options(viewport, options);
}
