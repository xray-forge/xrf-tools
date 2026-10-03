use tauri::State;
use xrf_renderer::{RenderSelection, RenderViewportId};

use crate::plugins::render::state::RenderState;

/// Mark what of one viewport's level is selected, or nothing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "set_selection"))]
#[tauri::command(rename = "set_selection")]
pub fn render_set_selection(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
  selection: Option<RenderSelection>,
) {
  state.renderer.set_selection(viewport, selection);
}
